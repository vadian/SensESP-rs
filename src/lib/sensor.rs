use eyeball::{shared::Observable, Subscriber};

use std::time::{Duration, SystemTime};

pub trait Sensor<T> {
    fn attach(&mut self) -> Subscriber<T>;
    fn tick(&mut self);
}

pub struct ConstantSensor<T> {
    observable: Observable<T>,
    duration: Duration,
    last_measurement: SystemTime,
    value: T,
}

impl<T: Copy> ConstantSensor<T> {
    pub fn new(val: T, duration: Duration) -> Self {
        let observable = Observable::new(val);
        let last_measurement = SystemTime::now();
        ConstantSensor::<T> {
            observable,
            value: val,
            duration,
            last_measurement,
        }
    }
}

impl<T: Copy> Sensor<T> for ConstantSensor<T> {
    fn tick(&mut self) {
        let now = SystemTime::now();

        match now.duration_since(self.last_measurement) {
            Ok(d) => match d >= self.duration {
                true => {
                    self.observable.set(self.value);
                    self.last_measurement = now;
                    ()
                }
                false => (),
            },
            Err(e) => log::error!("System time error on SensESP-rs tick: {:?}", e),
        }
    }

    fn attach(&mut self) -> Subscriber<T> {
        self.observable.subscribe()
    }
}
