#![feature(file_create_new)]

mod config;
mod device;
mod thread;

use std::fs::File;

use tokio::{join, runtime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("comb")?;
    let config_path = xdg_dirs.place_config_file("config.toml")?;

    let _ = File::create_new(&config_path);

    let rt = runtime::Runtime::new()?;

    rt.block_on(async {
        let config_watch_handle = config::watch(config_path)?;
        let device_watch_handle = device::watch()?;

        let _ = join!(config_watch_handle, device_watch_handle);

        Result::<(), notify::Error>::Ok(())
    })?;

    Ok(())
}
