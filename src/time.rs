use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ops::{Add, AddAssign};

use mursten::{Data, Updater};

pub type Time = SystemTime;

pub const CREATION_TIME: Time = UNIX_EPOCH;

#[derive(Debug)]
pub struct Clock {
    time: Time,
    delta: Duration,
    system_time: Time,
    system_delta: Duration,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            time: CREATION_TIME,
            delta: Duration::new(0, 0),
            system_time: Time::now(),
            system_delta: Duration::new(0, 0),
        }
    }
    pub fn system_time(&self) -> Time {
        self.system_time
    }
    pub fn system_delta(&self) -> Duration {
        self.system_delta
    }
    pub fn time(&self) -> Time {
        self.time
    }
    pub fn delta(&self) -> Duration {
        self.delta
    }
}

impl Clock {
    pub fn system_time_in_sec(&self) -> f32 {
        let d = self.system_time.duration_since(CREATION_TIME).unwrap();
        d.as_secs() as f32 + d.subsec_millis() as f32 / 1000.0
    }
    pub fn time_in_sec(&self) -> f32 {
        let d = self.time.duration_since(CREATION_TIME).unwrap();
        d.as_secs() as f32 + d.subsec_millis() as f32 / 1000.0
    }
    pub fn delta_as_sec(&self) -> f32 {
        self.delta.as_secs() as f32 + self.delta.subsec_millis() as f32 / 1000.0
    }
}


impl Add<Tick> for Clock {
    type Output = Clock;
    fn add(self, tick: Tick) -> Clock {
        Clock {
            system_delta: tick.system_time.duration_since(self.system_time).unwrap(),
            system_time: tick.system_time,
            time: self.time + tick.delta,
            delta: tick.delta,
        }
    }
}

impl AddAssign<Tick> for Clock {
    fn add_assign(&mut self, tick: Tick) {
        *self = Clock {
            system_delta: tick.system_time.duration_since(self.system_time).unwrap(),
            system_time: tick.system_time,
            time: self.time + tick.delta,
            delta: tick.delta,
        };
    }
}

#[derive(Debug)]
pub struct Tick {
    system_time: Time,
    delta: Duration,
}

pub trait OnTick {
    fn on_tick(&mut self, tick: Tick);
}

pub struct ClockUpdater {
    last_system_time: Time,
}

impl ClockUpdater {
    pub fn new() -> ClockUpdater {
        ClockUpdater {
            last_system_time: CREATION_TIME,
        }
    }
}


impl<B, D> Updater<B, D> for ClockUpdater
where
    D: Data + OnTick,
{
    fn update(&mut self, _: &mut B, data: &mut D) {
        let system_time = SystemTime::now();
        let delta = if self.last_system_time == CREATION_TIME {
            Duration::new(0, 0)
        } else {
            system_time.duration_since(self.last_system_time).unwrap()
        };

        let tick = Tick { system_time, delta };
        data.on_tick(tick);
        self.last_system_time = system_time;
    }
}
