use eyeball::{shared::Observable, Subscriber};
use std::time::{Duration, SystemTime};

pub trait SensESPSensor {
    fn tick(&mut self);
}

pub trait Attachable<T> {
    fn attach(&mut self) -> Subscriber<T>;
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

impl<T: Copy> SensESPSensor for ConstantSensor<T> {
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
}

impl<T: Copy> Attachable<T> for ConstantSensor<T> {
    fn attach(&mut self) -> Subscriber<T> {
        self.observable.subscribe()
    }
}

pub struct TimedSensor<T, F>
where
    T: Copy,
    F: Fn() -> T,
{
    observable: Observable<T>,
    duration: Duration,
    last_measurement: SystemTime,
    func: F,
}

impl<T, F> TimedSensor<T, F>
where
    T: Copy,
    F: Fn() -> T,
{
    pub fn new(func: F, duration: Duration) -> Self {
        let val = func();
        let observable = Observable::new(val.to_owned());
        let last_measurement = SystemTime::now() - duration;
        TimedSensor::<T, F> {
            observable,
            func,
            duration,
            last_measurement,
        }
    }
}

impl<T, F> Attachable<T> for TimedSensor<T, F>
where
    T: Copy,
    F: Fn() -> T,
{
    fn attach(&mut self) -> Subscriber<T> {
        self.observable.subscribe()
    }
}

impl<T, F> SensESPSensor for TimedSensor<T, F>
where
    T: Copy,
    F: Fn() -> T,
{
    fn tick(&mut self) {
        let now = SystemTime::now();

        match now.duration_since(self.last_measurement) {
            Ok(d) => match d >= self.duration {
                true => {
                    let val = (self.func)();
                    self.observable.set(val);
                    self.last_measurement = now;
                    ()
                }
                false => (),
            },
            Err(e) => log::error!("System time error on SensESP-rs tick: {:?}", e),
        }
    }
}
