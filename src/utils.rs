use std::{env, error::Error, fs::File, io::BufReader, path::Path};

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
