#![warn(
    clippy::all,
    // clippy::restriction,
     clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use jack_notification_rs::jack_notification_handle;
use std::{
    env,
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

fn maybe_name() -> Option<String> {
    let name = env::current_exe()
        .as_ref()
        .ok()?
        .file_name()?
        .to_str()?
        .to_owned();
    Some(name)
}

fn get_name() -> String {
    let default = "jack_notification_rs".to_owned();
    maybe_name().unwrap_or(default)
}

fn main() -> Result<(), Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    let name = get_name();

    let (_active_client, receiver) = jack_notification_handle(&name)?;

    while running.load(Ordering::Relaxed) {
        while let Ok(notification) = receiver.try_recv() {
            println!("{notification}");
        }
        sleep(Duration::from_millis(100));
    }

    Ok(())
}
