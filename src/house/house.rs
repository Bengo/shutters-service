use log::{debug, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use tokio::sync::Mutex; // Remplace std::sync::Mutex
use chrono::{Timelike, Utc};
use chrono_tz::Europe::Paris;
use sunrise::{Coordinates, SolarDay, SolarEvent};
use crate::Shutter;
use crate::weather::openweather::WeatherData;

const LAT: f64 = 48.43000;
const LON: f64 = -4.63;

#[derive(Debug, PartialEq)]
pub enum HouseMode {
   Auto,
   Absence,
   Party
}



pub struct House {
    pub pdv: Shutter,
    pub chambre_bas: Shutter,
    pub chambre_haut: Shutter,
    already_wheater_operated: AtomicBool,
    already_sunset_operated: AtomicBool,
    already_sunrise_operated: AtomicBool,
    pub mode: Arc<Mutex<HouseMode>>
}

impl House {
    pub fn new(pdv: Shutter, chambre_bas: Shutter, chambre_haut: Shutter, mode: Arc<Mutex<HouseMode>>) -> Self {
        House { pdv, chambre_bas, chambre_haut, already_wheater_operated: AtomicBool::new(false), already_sunset_operated: AtomicBool::new(false), already_sunrise_operated: AtomicBool::new(false), mode }
    }

    pub async fn close_with_sun(&self, running_loop: Arc<AtomicBool>) {
        while running_loop.load(Ordering::SeqCst) {
            // Obtenir l'heure UTC actuelle
            let utc_now = Utc::now();
            // Convertir en heure française (Europe/Paris)
            let paris_time = utc_now.with_timezone(&Paris);
            
            let coord = Coordinates::new(LAT, LON).unwrap();
            let solar_day = SolarDay::new(coord, paris_time.naive_utc().date());
            
            let sunset_time = solar_day.event_time(SolarEvent::Sunset)
                .map(|dt| dt.with_timezone(&Paris).time());
            let sunset_close = sunset_time.map(|time| time + chrono::Duration::minutes(15));

            if sunset_close.is_some() && sunset_close.unwrap().hour() == paris_time.hour() && sunset_close.unwrap().minute() <= paris_time.minute() && !self.already_sunset_operated.load(Ordering::SeqCst) {
                let current_mode = self.mode.lock().await;
                info!("Sunset time reached, closing all shutters for mode   {:?}", *current_mode);
                
                if *current_mode != HouseMode::Party {
                    self.pdv.close().await;
                }
                 
                self.chambre_bas.close().await;
                self.chambre_haut.close().await;
                self.already_sunset_operated.store(true, Ordering::SeqCst);
            } else {
                if self.already_sunset_operated.load(Ordering::SeqCst) && paris_time.hour() == 0 && paris_time.minute() <= 9 {
                    self.already_sunset_operated.store(false, Ordering::SeqCst);
                    debug!("Already operated for sunset, reset for next day: {:?} {:?}", sunset_close, paris_time);
                } else {
                    debug!("Not sunset time yet: {:?} {:?} or already operated {:?}", sunset_close, paris_time, self.already_sunset_operated.load(Ordering::SeqCst));
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5*60)).await;
        }
    }

    pub async fn open_with_sun(&self, running_loop: Arc<AtomicBool>) {
        while running_loop.load(Ordering::SeqCst) {
            // Obtenir l'heure UTC actuelle
            let utc_now = Utc::now();
            // Convertir en heure française (Europe/Paris)
            let paris_time = utc_now.with_timezone(&Paris);
            
            let coord = Coordinates::new(LAT, LON).unwrap();
            let solar_day = SolarDay::new(coord, paris_time.naive_utc().date());
            
            let sunrise_time = solar_day.event_time(SolarEvent::Sunrise)
                .map(|dt| dt.with_timezone(&Paris).time());
            let sunrise_open = sunrise_time.map(|time| time + chrono::Duration::minutes(15));

            if sunrise_open.is_some() && sunrise_open.unwrap().hour() == paris_time.hour() && sunrise_open.unwrap().minute() <= paris_time.minute() && !self.already_sunrise_operated.load(Ordering::SeqCst) {
                info!("Sunrise time reached, opening all shutters for mode   {:?}", self.mode);

                let current_mode = self.mode.lock().await;
                if *current_mode != HouseMode::Party {
                    self.pdv.open().await;
                    if *current_mode == HouseMode::Absence {
                        self.chambre_bas.open().await;
                        self.chambre_haut.open().await;
                    }
                }
                self.already_sunrise_operated.store(true, Ordering::SeqCst);
            } else {
                if self.already_sunrise_operated.load(Ordering::SeqCst) && paris_time.hour() == 0 && paris_time.minute() <= 9 {
                    self.already_sunrise_operated.store(false, Ordering::SeqCst);
                    debug!("Already operated for sunrise, reset for next day: {:?} {:?}", sunrise_open, paris_time);
                } else {
                    debug!("Not sunrise time yet: {:?} {:?} or already operated {:?}", sunrise_open, paris_time, self.already_sunrise_operated.load(Ordering::SeqCst));
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5*60)).await;

        }
    }

    pub async fn check_weather_and_operate_shutter(&self, weather_response: Arc<Mutex<WeatherData>>,  running_loop: Arc<AtomicBool>) {
       while running_loop.load(Ordering::SeqCst) {
            // Obtenir l'heure UTC actuelle
            let utc_now = Utc::now();
            // Convertir en heure française (Europe/Paris)
            let paris_time = utc_now.with_timezone(&Paris);

            // Here you would call your weather fetching function and decide whether to open or close the shutter
            // For example:
            let wr = weather_response.lock().await;
             debug!("Checking weather conditions at {:02}:{:02} with data: {:?}", paris_time.hour(), paris_time.minute(), *wr);
             // If it's between 10:00 and 18:00, the temperature is above 18 degrees, the wind speed is below 30 km/h and the cloudiness is below 75%, open the shutter to the middle position
            if paris_time.hour() >= 10 && paris_time.hour() < 18 && !self.already_wheater_operated.load(Ordering::SeqCst) && wr.temperature >= 18.0 && wr.wind_speed < 30.0 && wr.clouds < 75 {
                info!("Conditions obtenues pour passer les volets de la piece de vie et des chambres du haut en position intermediaire: {:?}", *wr);
                self.pdv.middle().await; // Open the shutter to the middle position
                self.chambre_haut.middle().await; // Open the shutter to the middle position
                self.already_wheater_operated.store(true, Ordering::SeqCst);
            } else {
                debug!("Conditions not met for closing mid the shutter: {:?} {:?}",*wr, self.already_wheater_operated.load(Ordering::SeqCst));
            }

            if paris_time.hour() == 18 && paris_time.minute() <= 9 && self.already_wheater_operated.load(Ordering::SeqCst) {
                info!("Resetting weather operation flag at 18:00: {:?}", weather_response.lock().await);
                self.already_wheater_operated.store(false, Ordering::SeqCst);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5*60)).await;
        }
    }


    pub async fn set_mode(&self, new_mode: HouseMode) {
        let mut current_mode = self.mode.lock().await;
        info!("Changing house mode from {:?} to {:?}", *current_mode, new_mode);
        *current_mode = new_mode;
        
    }
}  