use eyeball::shared::Observable;
use anyhow::Result;
use smol::{future, prelude::*, Async, io};

use std::time::{Duration, Instant};
use timerfd::{SetTimeFlags, TimerFd, TimerState};

pub struct ConstantSensor<T> {
    observable: Observable<T>
}

impl<T: Clone> ConstantSensor<T> {
    pub fn new(val: T) -> ConstantSensor<T> {
        let observable = Observable::new(val);
        ConstantSensor::<T> { observable }
    }

    /// Sleeps using an OS timer.
    async fn sleep(dur: Duration) -> io::Result<()> {
        // Create an OS timer.
        let mut timer = TimerFd::new()?;
        timer.set_state(TimerState::Oneshot(dur), SetTimeFlags::Default);

        // When the OS timer fires, a 64-bit integer can be read from it.
        Async::new(timer)?
            .read_with(|t| rustix::io::read(t, &mut [0u8; 8]).map_err(io::Error::from))
            .await?;
        Ok(())
    }
}