use chrono::{DateTime, Utc};
use keyframe::{ease_with_scaled_time, functions::EaseInOut};
use std::{borrow::Cow, collections::HashMap, time::Duration};

pub struct Animator {
    animations: HashMap<String, AnimationInner>,
    remove_later: Vec<String>,
}

#[derive(Debug)]
struct AnimationInner {
    start: f64,
    end: f64,
    current: f64,

    start_time: DateTime<Utc>,
    duration: Duration,
    t: AnimationType,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum AnimationType {
    Single,
    Repeat,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            animations: HashMap::default(),
            remove_later: vec![],
        }
    }

    pub fn add_animation(
        &mut self,
        name: Cow<'_, str>,
        start: f64,
        end: f64,
        duration: Duration,
        t: AnimationType,
    ) {
        match self.animations.entry(name.into_owned()) {
            std::collections::hash_map::Entry::Occupied(o) =>
                panic!("Add already existed animation with {}. It is a bug. Please submit a bug to https://github.com/l4l/yofi/issues", o.key()),
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(AnimationInner::new(start, end, duration, t));
            }
        }
    }

    pub fn cancel_animation(&mut self, name: &str) {
        self.animations.remove(name);
    }

    pub fn add_step_animation(
        &mut self,
        name: Cow<'_, str>,
        start: f64,
        end: f64,
        step_duration: Duration,
        t: AnimationType,
    ) {
        let full_duration = Duration::from_millis(
            ((end - start) as u128 * step_duration.as_millis())
                .try_into()
                .unwrap_or(u64::MAX),
        );
        self.add_animation(name, start, end, full_duration, t);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.animations.contains_key(name)
    }

    pub fn get_value(&mut self, name: &str) -> Option<f64> {
        Some(self.animations.get(name)?.current)
    }

    pub fn proceed(&mut self) -> bool {
        let current_time = Utc::now();

        for name in std::mem::take(&mut self.remove_later).into_iter() {
            let animation = self.animations.remove(&name).unwrap();
            if animation.t == AnimationType::Repeat {
                self.animations.insert(
                    name.clone(),
                    AnimationInner::new(
                        animation.start,
                        animation.end,
                        animation.duration,
                        animation.t,
                    ),
                );
            }
        }

        for (name, animation) in self.animations.iter_mut() {
            let time = (current_time - animation.start_time).num_milliseconds() as f64;

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
    fn new(start: f64, end: f64, duration: Duration, t: AnimationType) -> Self {
        Self {
            start,
            end,
            current: start,
            start_time: Utc::now(),
            duration,
            t,
        }
    }
}
