use std::{
    env,
    error::Error,
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, FixedOffset, Utc};
use tokio::time::Instant;

use crate::app_state::AppState;

pub fn get_app_state() -> Result<AppState, Box<dyn Error>> {
    let potential_settings_path = vec![
        env::current_exe()?.join("settings.json"),
        env::current_dir()?.join("settings.json"),
    ];

    let settings_path = potential_settings_path
        .iter()
        .find(|path| Path::new(path).exists())
        .ok_or(format!(
            "settings.json not found at paths: {:#?}",
            potential_settings_path
        ))?;

    let file = File::open(settings_path)?;
    let app_state: AppState = serde_json::from_reader(BufReader::new(file))?;
    Ok(app_state)
}

pub fn datetime_to_instant(dt: DateTime<FixedOffset>) -> Instant {
    let target_utc: DateTime<Utc> = dt.into();
    let now = SystemTime::now();
    let target_time: SystemTime = target_utc.into();

    let duration = target_time
        .duration_since(now)
        .unwrap_or(Duration::from_secs(0)); // Handle past times gracefully

    Instant::now() + duration
}

pub fn instant_to_datetime(instant: Instant) -> DateTime<FixedOffset> {
    let system_time = SystemTime::now() + (instant - Instant::now());
    let datetime: DateTime<Utc> = system_time.into();
    datetime.with_timezone(&FixedOffset::west_opt(0).unwrap())
}
