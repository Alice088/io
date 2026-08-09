/// System events published by components to the event bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Satellite booted / restarted.
    Boot,

    /// Solar panels / antenna deployment started.
    Deploy,

    /// Battery dropped to/below LOW_PERCENT.
    BatteryLow,

    /// Battery dropped to/below CRITICAL_PERCENT.
    BatteryCritical,

    /// Battery recovered above the critical threshold.
    BatteryRestored,

    /// Gyro read failed.
    GyroFault,

    /// Gyro reads OK again.
    GyroRestored,

    /// Link to the game world lost.
    LinkLost,

    /// Link to the game world back.
    LinkRestored,

    /// Watchdog deadline missed.
    WatchdogTimeout,
}
