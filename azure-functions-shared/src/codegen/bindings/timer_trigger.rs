use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

pub const TIMER_TRIGGER_TYPE: &str = "timerTrigger";

#[derive(Debug, Clone)]
pub struct TimerTrigger {
    pub name: Cow<'static, str>,
    pub schedule: Option<Cow<'static, str>>,
    pub run_on_startup: Option<bool>,
    pub use_monitor: Option<bool>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for TimerTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", TIMER_TRIGGER_TYPE)?;
        map.serialize_entry("direction", "in")?;

        if let Some(schedule) = self.schedule.as_ref() {
            map.serialize_entry("schedule", schedule)?;
        }
        if let Some(run_on_startup) = self.run_on_startup.as_ref() {
            map.serialize_entry("runOnStartup", run_on_startup)?;
        }
        if let Some(use_monitor) = self.use_monitor.as_ref() {
            map.serialize_entry("useMonitor", use_monitor)?;
        }

        map.end()
    }
}
