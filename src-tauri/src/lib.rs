// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub async fn scan_dir(
    dir: &str,
    tx_dir: tokio::sync::mpsc::Sender<Vec<String>>,
    tx_file: tokio::sync::mpsc::Sender<Vec<String>>,
) {
    for entry in walkdir::WalkDir::new(dir) {
        if let Ok(entry) = entry {
            let path = entry.path();
            let path_vec: Vec<_> = path
                .to_string_lossy()
                .split('/')
                .map(|s| s.to_string())
                .collect();
            if path.is_dir() {
                if let Err(e) = tx_dir.send(path_vec).await {
                    log::error!("tx_dir.send err: {}", e);
                }
            } else if path.is_file() {
                if let Err(e) = tx_file.send(path_vec).await {
                    log::error!("tx_file.send err: {}", e);
                }
            } else {
                log::info!("not dir and file: {}", entry.path().display());
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // cargo test tests::scan_dir_test -- --nocapture
    #[tokio::test]
    async fn scan_dir_test() {
        let _ = tracing_subscriber::fmt()
            .with_line_number(true)
            .with_ansi(false)
            .try_init();
        let (tx_dir, mut rx_dir) = tokio::sync::mpsc::channel(100);
        let (tx_file, mut rx_file) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            scan_dir("./src", tx_dir, tx_file).await;
        });
        while let Some(path_vec) = rx_dir.recv().await {
            log::info!("dir: {}", path_vec.join("/"));
        }
        while let Some(path_vec) = rx_file.recv().await {
            log::info!("file: {}", path_vec.join("/"));
        }
    }
}
