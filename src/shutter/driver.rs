use rppal::gpio::Gpio;
use log::{debug};


pub struct Shutter {
    number: u8
}

impl Shutter {
    pub fn new(number: u8) -> Self {
        Shutter { number }
    }
    pub async fn open(&self)  {     
        // Code to open the shutter (simulate GPIO operation)
        info!("Opening shutter on GPIO pin {}", self.number);    
        for _ in 0..3 {
            self.impulse(100).await;
        }
    }

    pub async fn close(&self) {
        // Code to close the shutter (simulate GPIO operation)
        info!("Closing shutter on GPIO pin {}", self.number);    
        for _ in 0..4 {
            self.impulse(100).await;
        }
    }

    pub async fn stop(&self) {
        // Code to stop the shutter (simulate GPIO operation)
        info!("Stopping shutter on GPIO pin {}", self.number);    
        self.impulse(100).await;
    }

    pub async fn reset(&self) {
        // Code to reset the shutter (simulate GPIO operation)
        info!("Resetting shutter on GPIO pin {}", self.number); 
        for _ in 0..25 {
            self.impulse(100).await;
        }   
    }

    pub async fn middle(&self) {
        // Code to set the shutter to the middle position (simulate GPIO operation)
        info!("Setting shutter to middle position on GPIO pin {}", self.number);    
        self.open().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(180000)).await;
        self.close().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(8800)).await;
        self.stop().await;
    }

    async fn impulse(&self, duration: u64) {
        // Code to set the GPIO pin high for a specified duration (in milliseconds) and then set it low
        // Initialise le GPIO
        let gpio = Gpio::new();
    
        // Configure la broche number en sortie
        let mut pin = gpio.expect("REASON").get(self.number).expect("REASON").into_output(); // Configure la broche number en sortie

        // Allume la LED, attend 1 seconde, puis l'éteint
        pin.set_high();
        tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;
        pin.set_low();
        tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;

    }
}