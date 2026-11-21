#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(err) = tauri::Builder::default().run(tauri::generate_context!()) {
        eprintln!("LogLens failed to start: {err}");
        std::process::exit(1);
    }
}
