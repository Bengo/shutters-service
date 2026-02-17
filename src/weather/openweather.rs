use std::sync::{Arc, Mutex};
use openweather_sdk::{OpenWeather, Units, Language};
use chrono::{Timelike, Utc};
use chrono_tz::Europe::Paris;
use sunrise::{Coordinates, SolarDay, SolarEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use log::{debug};


pub struct WeatherData {
    // Define the structure of the weather data you want to store
    // For example:
    pub temperature: f64,
    pub wind_speed: f64,
    pub clouds:i64
}

impl std::fmt::Debug for WeatherData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeatherData")
            .field("temperature", &self.temperature)
            .field("wind_speed", &self.wind_speed)
            .field("clouds", &self.clouds)
            .finish()
    }
}

pub async fn get_weather(weather_response: Arc<Mutex<WeatherData>>) {
    let openweather = OpenWeather::new(
        std::env::var("OPENWEATHER_API_KEY").unwrap(),
        Units::Metric,
        Language::English,
    );

    // Saint renan
    let lat = std::env::var("LATITUDE").unwrap_or_else(|_| "48.43".to_string()).parse::<f64>().unwrap_or(48.43);
    debug!("Using latitude: {}", lat);
    let lon = std::env::var("LONGITUDE").unwrap_or_else(|_| "-4.63".to_string()).parse::<f64>().unwrap_or(-4.63);
    debug!("Using longitude: {}", lon);

    let res = openweather.current.call(lat, lon).await;
    debug!("Weather response: {res:?}");
    let current_weather = res.as_ref().unwrap();
    // Store the response in the shared variable
    let mut response = weather_response.lock().unwrap();
    *response = WeatherData {
        temperature: current_weather.main.temp,
        wind_speed: current_weather.wind.speed,
        clouds: current_weather.clouds.all
    };
}

pub async fn schedule_hourly_between_sunrise_sunset(weather_response: Arc<Mutex<WeatherData>>, running_loop: Arc<AtomicBool>) {
       
        while running_loop.load(Ordering::SeqCst) {
 
            // Obtenir l'heure UTC actuelle
            let utc_now = Utc::now();
            // Convertir en heure française (Europe/Paris)
            let paris_time = utc_now.with_timezone(&Paris);

            let lat = std::env::var("LATITUDE").unwrap_or_else(|_| "48.43".to_string()).parse::<f64>().unwrap_or(48.43);
            let lon = std::env::var("LONGITUDE").unwrap_or_else(|_| "-4.63".to_string()).parse::<f64>().unwrap_or(-4.63);

            let coord = Coordinates::new(lat, lon).unwrap();
            let solar_day = SolarDay::new(coord, paris_time.naive_utc().date());
            
            let sunrise_time = solar_day.event_time(SolarEvent::Sunrise)
                .map(|dt| dt.with_timezone(&Paris).time());
            let sunset_time = solar_day.event_time(SolarEvent::Sunset)
                .map(|dt| dt.with_timezone(&Paris).time());
            
            debug!("Sunrise: {:?}, Sunset: {:?}", sunrise_time, sunset_time);
            
            // Si on est entre le lever et le coucher du soleil
            if let (Some(sunrise), Some(sunset)) = (sunrise_time, sunset_time) {
                if paris_time.time() >= sunrise && paris_time.time() < sunset {
                    get_weather(weather_response.clone()).await;
                    
                    // Attendre jusqu'à l'heure suivante
                    let next_hour = (paris_time.hour() + 1) % 24;
                    let minutes_to_wait = 60 - paris_time.minute();
                    let seconds_to_wait = 60 - paris_time.second();
                    
                    debug!("Next execution at {}:00", next_hour);
                    tokio::time::sleep(
                        tokio::time::Duration::from_secs(
                            (minutes_to_wait as u64 * 60) + seconds_to_wait as u64
                        )
                    ).await;
                } else {
                    // Si on est hors des heures de jour, attendre 30 minutes et revérifier
                    debug!("Outside daylight hours, checking again in 30 minutes");
                    tokio::time::sleep(tokio::time::Duration::from_secs(30 * 60)).await;
                }
            }

        }

}
