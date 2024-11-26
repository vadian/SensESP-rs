use crate::sensor::Sensor;

pub struct Application {
    sensors: Vec<Box<dyn Sensor>>,
}

impl Application {
    pub fn new() -> Self {
        Application {
            sensors: Vec::new(),
        }
    }
    pub fn register(mut self, s: impl Sensor + 'static) -> Self {
        self.sensors.push(Box::new(s));
        self
    }

    pub fn tick(&mut self) {
        for s in &mut self.sensors {
            s.as_mut().tick();
        }
    }
}
