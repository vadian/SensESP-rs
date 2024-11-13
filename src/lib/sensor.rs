use eyeball::shared::Observable;
use anyhow::Result;
use smol::{future, prelude::*, Async};

pub struct ConstantSensor<T> {
    observable: Observable<T>
}

impl<T: Clone> ConstantSensor<T> {
    pub fn new(val: T) -> ConstantSensor<T> {
        let observable = Observable::new(val);
        ConstantSensor::<T> { observable }
    }
}