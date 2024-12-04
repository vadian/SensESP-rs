use crate::sensor::SensESPSensor;

pub struct Application {
    sensors: Vec<Box<dyn SensESPSensor>>,
}

impl Application {
    pub fn new() -> Self {
        Application {
            sensors: Vec::new(),
        }
    }
    pub fn register(mut self, s: impl SensESPSensor + 'static) -> Self {
        self.sensors.push(Box::new(s));
        self
    }

    pub fn tick(&mut self) {
        for s in &mut self.sensors {
            s.as_mut().tick();
        }
    }
}
