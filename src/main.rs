//! jack notifications for rust.
//!
//! The jack crate exposes the callbacks to rust that make it possible to listen to jack notifications.
//! This library implements a `enum` for the possible notifications.
//! A simple jack client that is listens to those notifications and send them through a channel.
//! An example application prints those `Notifications` using the `Display` trait to `stdout`.
//!
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // from clippy::restriction
    clippy::blanket_clippy_restriction_lints,
    renamed_and_removed_lints,
    single_call_fn,
    clippy::question_mark_used,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::std_instead_of_alloc,
    reason = "i think it is ok"
)]

use core::{
    error::Error,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use jack_notifications_rs::{jack_notification_handle, Notification};
use std::thread::sleep;
use std::{env, sync::Arc};

/// .
fn maybe_name() -> Option<String> {
    let name = env::current_exe()
        .as_ref()
        .ok()?
        .file_name()?
        .to_str()?
        .to_owned();
    Some(name)
}

/// .
fn get_name() -> String {
    let default = "jack_notification_rs".to_owned();
    maybe_name().unwrap_or(default)
}

fn main() -> Result<(), Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let run_ref = Arc::clone(&running);

    ctrlc::set_handler(move || {
        run_ref.store(false, Ordering::Relaxed);
    })?;
    let name = get_name();

    let (_active_client, receiver) = jack_notification_handle::<Notification>(&name)?;

    while running.load(Ordering::Relaxed) {
        while let Ok(notification) = receiver.try_recv() {
            println!("{notification}");
        }
        sleep(Duration::from_millis(100));
    }

    Ok(())
}
