use serde::{ser::SerializeMap, Serialize, Serializer};

#[doc(hidden)]
pub struct TimerTrigger {
    pub name: &'static str,
    pub schedule: &'static Option<&'static str>,
    pub run_on_startup: &'static Option<bool>,
    pub use_monitor: &'static Option<bool>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for TimerTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", self.name)?;
        map.serialize_entry("type", "timerTrigger")?;
        map.serialize_entry("direction", "in")?;
        if self.schedule.is_some() {
            map.serialize_entry("schedule", self.schedule.as_ref().unwrap())?;
        }
        if self.run_on_startup.is_some() {
            map.serialize_entry("runOnStartup", self.run_on_startup.as_ref().unwrap())?;
        }
        if self.use_monitor.is_some() {
            map.serialize_entry("useMonitor", self.use_monitor.as_ref().unwrap())?;
        }

        map.end()
    }
}
