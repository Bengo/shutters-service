use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

extern crate libsystemd;
use libsystemd::daemon::{self};

use log::{info, warn};

mod weather;
mod shutter;
mod house;
use shutter::driver::Shutter;
use house::house::House;

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
        // ensure at least one immediate fetch
        if scheduler_wr.lock().unwrap().temperature == 0.0 {
             info!("Performing initial weather fetch...");
             weather::openweather::get_weather(scheduler_wr.clone()).await;
        }
        info!("Scheduling weather fetcher ...");
        weather::openweather::schedule_hourly_between_sunrise_sunset(scheduler_wr, running_clone).await; 
    });



    let house = Arc::new(House::new(Shutter::new(18), Shutter::new(17), Shutter::new(27))); // Initialize a house with three shutters on GPIO pins 17, 18 and 19
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
        close_house.close_all_with_sun(close_running).await;

    });
  

    // Keep the main thread alive until 'running' becomes false
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_secs(10)).await;
        //debug!("Main loop alive, weather response: {:?}", weather_response.lock().unwrap());
    }
    info!("shutters-service stopping");
    Ok(())

}
