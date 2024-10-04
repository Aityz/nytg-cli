use crate::app::App;

pub fn load(data: String) -> Result<App, serde_json::Error> {
    let app: App = serde_json::from_str(&data)?;

    Ok(app)
}

pub fn save(app: App) -> Result<String, serde_json::Error> {
    let data = serde_json::to_string_pretty(&app)?;

    Ok(data)
}

pub fn get_loc() -> String {
    let home = std::path::PathBuf::from(
        std::env::var("HOME").expect("This program currently doesn't work on Windows"),
    );

    let config_dir = home.join(".config").join("nytg_cli");

    if !config_dir.exists() {
        let _ = std::fs::create_dir_all(&config_dir);
    }

    config_dir.join("state.json").to_string_lossy().to_string()
}
