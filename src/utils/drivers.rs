use tokio::process::{Command, Child};
use std::sync::Arc;
use anyhow::{anyhow, Result};
use tokio::sync::Mutex;

pub async fn start_chromedriver() -> Result<Child> {
    let path = std::env::var("CHROME_DRIVER_PATH")
        .map_err(|e| anyhow!("Failed to get CHROME_DRIVER_PATH: {:?}", e))?;

    let chromedriver_process = Command::new(path)
        .arg("--port=4450")
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow!("Failed to start ChromeDriver: {:?}", e))?;

    println!("ChromeDriver started on port 4450");
    Ok(chromedriver_process)
}

pub async fn shutdown_chromedriver(chromedriver_process: Arc<Mutex<Child>>) {
    let mut process = chromedriver_process.lock().await;

    if let Ok(Some(_status)) = process.try_wait() {
        println!("ChromeDriver process already terminated");
    } else {
        if let Err(e) = process.kill().await {
            eprintln!("Failed to kill ChromeDriver: {:?}", e);
        } else {
            println!("ChromeDriver process terminated");
        }
    }
}