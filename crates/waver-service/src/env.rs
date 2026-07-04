use std::time::Duration;

const ENV_WAVER_TICK_DELAY_MS: &str = "WAVER_TICK_DELAY_MS";
const ENV_WAVER_TICK_ERROR_DELAY_MS: &str = "WAVER_TICK_ERROR_DELAY_MS";

pub fn tick_delay() -> Duration {
    std::env::var(ENV_WAVER_TICK_DELAY_MS)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(default_tick_delay)
}

pub fn tick_error_delay() -> Duration {
    std::env::var(ENV_WAVER_TICK_ERROR_DELAY_MS)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(default_tick_error_delay)
}

pub fn default_tick_delay() -> Duration {
    Duration::from_millis(100)
}

pub fn default_tick_error_delay() -> Duration {
    Duration::from_millis(500)
}
