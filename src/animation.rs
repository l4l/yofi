use keyframe::{ease_with_scaled_time, functions::EaseInOut};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct Animator {
    animations: HashMap<Animation, AnimationInner>,
    remove_later: Vec<Animation>,
}

#[derive(Debug)]
struct AnimationInner {
    start: f64,
    end: f64,
    current: f64,

    start_time: Instant,
    duration: Duration,
}

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum Animation {
    Height,
}

pub struct AnimationConfig {
    pub height: Duration,
}

impl Animator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_animation(&mut self, name: Animation, start: f64, end: f64, duration: Duration) {
        match self.animations.entry(name) {
            std::collections::hash_map::Entry::Occupied(o) =>
                panic!("Add already existed animation with {:?}. It is a bug. Please submit a bug to https://github.com/l4l/yofi/issues", o),
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(AnimationInner::new(start, end, duration));
            }
        }
    }

    pub fn cancel_animation(&mut self, name: Animation) {
        self.animations.remove(&name);
    }

    pub fn contains(&self, name: Animation) -> bool {
        self.animations.contains_key(&name)
    }

    pub fn get_value(&mut self, name: Animation) -> Option<f64> {
        Some(self.animations.get(&name)?.current)
    }

    pub fn proceed(&mut self) -> bool {
        let current_time = Instant::now();

        for name in std::mem::take(&mut self.remove_later).into_iter() {
            self.animations.remove(&name).unwrap();
        }

        for (name, animation) in self.animations.iter_mut() {
            let time = (current_time - animation.start_time).as_millis() as f64;

            let mut start = animation.start;
            let mut end = animation.end;

            let flip = if start >= end {
                std::mem::swap(&mut start, &mut end);
                true
            } else {
                false
            };

            let mut value = ease_with_scaled_time(
                EaseInOut,
                start,
                end,
                time,
                animation.duration.as_millis() as f64,
            );

            if value >= end {
                self.remove_later.push(name.clone());
            }

            if flip {
                let delta = value - start;
                value = end - delta;
            }

            animation.current = value;
        }

        !self.animations.is_empty()
    }

    // Minimum time of proceed animation in event loop
    pub fn proceed_step(&self) -> Option<Duration> {
        if self.animations.is_empty() {
            None
        } else {
            Some(Duration::from_millis(100))
        }
    }
}

impl AnimationInner {
    fn new(start: f64, end: f64, duration: Duration) -> Self {
        Self {
            start,
            end,
            current: start,
            start_time: Instant::now(),
            duration,
        }
    }
}
