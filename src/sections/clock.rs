use iced::widget::mouse_area;

use crate::components::section;

#[derive(Debug, Clone, Copy)]
enum TimeFormat {
    _12h,
    _24h,
}

pub struct Clock {
    now: jiff::Zoned,
    format: TimeFormat,
    expanded: bool,
}

#[derive(Debug, Clone)]
pub enum ClockMessage {
    Tick(jiff::Zoned),
    ToggleExpanded,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            now: jiff::Zoned::now(),
            // TODO: make configurable
            format: TimeFormat::_12h,
            expanded: false,
        }
    }

    pub fn update(&mut self, message: ClockMessage) {
        match message {
            ClockMessage::Tick(local_time) => {
                self.now = local_time;
            }
            ClockMessage::ToggleExpanded => {
                self.expanded = !self.expanded;
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, ClockMessage> {
        let format = match (self.format, self.expanded) {
            // Sun 5:14 PM
            (TimeFormat::_12h, false) => "%a %l:%M %p",
            // Sunday, Jun 22 5:14:34 PM
            (TimeFormat::_12h, true) => "%A, %b %e %H:%M:%S %p",
            // Sun 22:14
            (TimeFormat::_24h, false) => "%a %k:%M",
            // Sunday, Jun 22 22:14:34
            (TimeFormat::_24h, true) => "%A, %b %e %k:%M:%S",
        };

        mouse_area(section(iced::widget::text(
            self.now.strftime(format).to_string(),
        )))
        .on_press(ClockMessage::ToggleExpanded)
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<ClockMessage> {
        time::every_second().map(|_| ClockMessage::Tick(jiff::Zoned::now()))
    }
}

pub mod time {
    use iced::{
        Subscription,
        advanced::subscription::{self, Hasher},
    };

    /// Returns an [`iced::Subscription`] that produces a message at every integer second
    /// transition.
    pub fn every_second() -> Subscription<std::time::Instant> {
        subscription::from_recipe(EverySecond())
    }

    struct EverySecond();

    impl subscription::Recipe for EverySecond {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut Hasher) {
            use std::hash::Hash;
            std::any::TypeId::of::<Self>().hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: subscription::EventStream,
        ) -> iced::futures::stream::BoxStream<'static, Self::Output> {
            use iced::futures::stream::StreamExt;

            let now = tokio::time::Instant::now();
            let now_sys = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time is valid");

            let subsec_nanos = now_sys.subsec_nanos() as u64;
            let until_next_second = std::time::Duration::from_secs(1)
                .checked_sub(std::time::Duration::from_nanos(subsec_nanos))
                .unwrap_or_default();

            let mut interval = tokio::time::interval_at(
                now + until_next_second,
                tokio::time::Duration::from_secs(1),
            );
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            let stream = iced::futures::stream::unfold(interval, |mut interval| async move {
                Some((interval.tick().await.into_std(), interval))
            });

            stream.boxed()
        }
    }
}
