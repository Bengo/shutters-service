use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use tokio::time::Duration;
use tokio::sync::Mutex; // Remplace std::sync::Mutex

extern crate libsystemd;
use libsystemd::daemon::{self};

use log::{info, warn};

mod weather;
mod shutter;
mod house;
use shutter::driver::Shutter;
use house::house::House;
use house::house::HouseMode;
use zbus::{connection, interface};


struct ShutterService {
    house: Arc<House>
}

#[interface(name = "fr.bengo.ShutterService")]
impl ShutterService {
    async fn change_mode(&mut self, new_mode: HouseMode) -> zbus::fdo::Result<()> {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        self.house.set_mode(new_mode).await;
        Ok(())
    }

    async fn open_all(&self) -> String {
        self.house.open_all().await;
        "All shutters opened".to_string()
    }
    
    async fn close_all(&self) -> String {
        self.house.close_all().await;
        "All shutters closed".to_string()
     }

     async fn middle_all(&self) -> String {
        self.house.middle_all().await;
        "All shutters set to middle position".to_string()
     }

     async fn open_pdv(&self) -> String {
        self.house.pdv.open().await;
        "Piece de vie ouverte".to_string()
     }

    async fn close_pdv(&self) -> String {
        self.house.pdv.close().await;
        "Piece de vie closed".to_string()
    }

    async fn middle_pdv(&self) -> String {
        self.house.pdv.middle().await;
        "Piece de vie set to middle position".to_string()
     }

     async fn open_chambre_bas(&self) -> String {
        self.house.chambre_bas.open().await;
        "Chambre bas ouverte".to_string()
     }

     async fn close_chambre_bas(&self) -> String {
        self.house.chambre_bas.close().await;
        "Chambre bas closed".to_string()
     }

     async fn middle_chambre_bas(&self) -> String {
        self.house.chambre_bas.middle().await;
        "Chambre bas set to middle position".to_string()
     }

     async fn open_chambre_haut(&self) -> String {
        self.house.chambre_haut.open().await;
        "Chambre haut ouverte".to_string()
     }

     async fn close_chambre_haut(&self) -> String {
        self.house.chambre_haut.close().await;
        "Chambre haut closed".to_string()
     }

     async fn middle_chambre_haut(&self) -> String {
        self.house.chambre_haut.middle().await;
        "Chambre haut set to middle position".to_string()
     }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("shutters-service starting");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    // Listen for Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for event");
        if r.swap(false, Ordering::SeqCst) {
            warn!("Shutdown signal received");
        }
    });

    // Shared storage for weather response (as in shutters-core)
    let weather_response = Arc::new(Mutex::new(weather::openweather::WeatherData {
        temperature: 0.0,
        wind_speed: 0.0,
        clouds: 0
    }));

    if !daemon::booted() {
        panic!("Not running systemd, early exit.");
    };


    // Spawn the weather scheduler in background (it loops internally)
    let scheduler_wr = weather_response.clone();
    let running_clone = running.clone();
    tokio::spawn(async move {
        // 1. On détermine si on doit fetcher SANS garder le lock ouvert
        let is_initial_data_empty = {
            let wr = scheduler_wr.lock().await;
            wr.temperature == 0.0 
        }; // Le garde 'wr' est officiellement détruit ici.

        // 2. Maintenant on peut faire des .await sans porter de verrou
        if is_initial_data_empty {
            info!("Performing initial weather fetch...");
            let weather_data = weather::openweather::get_weather().await;
            let mut wr = scheduler_wr.lock().await;
            *wr = weather_data;
        }

        info!("Scheduling weather fetcher ...");
        weather::openweather::schedule_hourly_between_sunrise_sunset(scheduler_wr.clone(), running_clone).await; 
    });


    let house_mode = Arc::new(Mutex::new(HouseMode::Auto));
    let house = Arc::new(House::new(Shutter::new(18), Shutter::new(17), Shutter::new(27), house_mode)); // Initialize a house with three shutters on GPIO pins 17, 18 and 19
    let shutter_service = ShutterService { house: house.clone() };
    let _conn = connection::Builder::session()?
        .name("fr.bengo.ShutterService")?
        .serve_at("/fr/bengo/ShutterService", shutter_service)?
        .build()
        .await?;
   
    let house_wr = weather_response.clone();
    let house_running = running.clone();
    let open_running = running.clone();
    let close_running = running.clone();
    let open_house = Arc::clone(&house);
    let close_house = Arc::clone(&house);
    let weather_house = Arc::clone(&house);
    tokio::spawn(async move {
        weather_house.check_weather_and_operate_shutter(house_wr, house_running).await;
    });
    tokio::spawn(async move {
        open_house.open_with_sun(open_running).await;

    });
    tokio::spawn(async move {
        close_house.close_with_sun(close_running).await;

    });

    // Keep the main thread alive until 'running' becomes false
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_secs(10)).await;
        //debug!("Main loop alive, weather response: {:?}", weather_response.lock().unwrap());
    }

    info!("shutters-service stopping");
    Ok(())

}
