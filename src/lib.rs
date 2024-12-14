//! jack notifications for rust.
//!
//! The jack crate exposes the callbacks to rust that make it possible to listen to jack notifications.
//! This library implements a `enum` for the possible notifications.
//! A simple jack client that is listens to those notifications and send them through a channel.
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
    // single_call_fn,
    clippy::question_mark_used,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::std_instead_of_alloc,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::pattern_type_mismatch,
    clippy::use_debug,
    clippy::min_ident_chars,
    clippy::missing_trait_methods,
    clippy::unwrap_used,
    reason = "i think it is ok"
)]

use core::{convert::From, fmt::Display, marker::Send};
use jack::Client;
use std::{
    fmt,
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Notification {
    ThreadInit,
    Shutdown(jack::ClientStatus, String),
    Freewheel(bool),
    SampleRate(jack::Frames),
    ClientRegistration(String, bool),
    PortRegistration(jack::PortId, bool),
    PortRename(jack::PortId, String, String),
    PortsConnected(jack::PortId, jack::PortId, bool),
    GraphReorder,
    XRun,
}

impl Display for Notification {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThreadInit => writeln!(f, "JACK: thread init"),
            Self::Shutdown(status, reason) => {
                writeln!(f, "JACK: shutdown with status {status:?} because {reason}")
            }
            Self::Freewheel(arg0) => writeln!(
                f,
                "JACK: freewheel mode is {}\n",
                if *arg0 { "on" } else { "off" }
            ),
            Self::SampleRate(srate) => writeln!(f, "JACK: sample rate changed to {srate}"),
            Self::ClientRegistration(name, is_reg) => writeln!(
                f,
                "JACK: {} client with name \"{}\"",
                if *is_reg {
                    "registered"
                } else {
                    "unregistered"
                },
                name
            ),
            Self::PortRegistration(port_id, is_reg) => writeln!(
                f,
                "JACK: {} port with id {}\n",
                if *is_reg {
                    "registered"
                } else {
                    "unregistered"
                },
                port_id
            ),
            Self::PortRename(port_id, old_name, new_name) => writeln!(
                f,
                "JACK: port with id {port_id} renamed from {old_name} to {new_name}\n"
            ),
            Self::PortsConnected(port_id_a, port_id_b, are_connected) => writeln!(
                f,
                "JACK: ports with id {} and {} are {}\n",
                port_id_a,
                port_id_b,
                if *are_connected {
                    "connected"
                } else {
                    "disconnected"
                }
            ),
            Self::GraphReorder => writeln!(f, "JACK: graph reordered"),
            Self::XRun => writeln!(f, "JACK: xrun occurred"),
        }
    }
}

pub struct DummyProcessHandler;

impl jack::ProcessHandler for DummyProcessHandler {
    #[inline]
    fn process(&mut self, _c: &jack::Client, _ps: &jack::ProcessScope) -> jack::Control {
        jack::Control::Continue
    }

    #[inline]
    fn buffer_size(&mut self, _c: &jack::Client, _size: jack::Frames) -> jack::Control {
        jack::Control::Continue
    }
}

pub struct SimpleNotificationHandler<T>
where
    T: From<Notification>,
{
    pub msg_sender: Sender<T>,
}

impl<T> jack::NotificationHandler for SimpleNotificationHandler<T>
where
    T: From<Notification> + Send,
{
    #[inline]
    fn thread_init(&self, _: &jack::Client) {
        let _ignored_result = self.msg_sender.send(Notification::ThreadInit.into());
    }

    #[inline]
    unsafe fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        let _ignored_result = self
            .msg_sender
            .send(Notification::Shutdown(status, reason.to_owned()).into());
    }

    #[inline]
    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        let _ignored_result = self
            .msg_sender
            .send(Notification::Freewheel(is_enabled).into());
    }

    #[inline]
    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        match self.msg_sender.send(Notification::SampleRate(srate).into()) {
            Ok(()) => jack::Control::Continue,
            Err(_) => jack::Control::Quit,
        }
    }

    #[inline]
    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        let _ignored_result = self
            .msg_sender
            .send(Notification::ClientRegistration(name.to_owned(), is_reg).into());
    }

    #[inline]
    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        let _ignored_result = self
            .msg_sender
            .send(Notification::PortRegistration(port_id, is_reg).into());
    }

    #[inline]
    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        match self.msg_sender.send(
            Notification::PortRename(port_id, old_name.to_owned(), new_name.to_owned()).into(),
        ) {
            Ok(()) => jack::Control::Continue,
            Err(_) => jack::Control::Quit,
        }
    }

    #[inline]
    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        let _ignored_result = self
            .msg_sender
            .send(Notification::PortsConnected(port_id_a, port_id_b, are_connected).into());
    }

    #[inline]
    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        match self.msg_sender.send(Notification::GraphReorder.into()) {
            Ok(()) => jack::Control::Continue,
            Err(_) => jack::Control::Quit,
        }
    }

    #[inline]
    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        match self.msg_sender.send(Notification::XRun.into()) {
            Ok(()) => jack::Control::Continue,
            Err(_) => jack::Control::Quit,
        }
    }
}

/// Return type of the `jack_notification_handle` function
type JackHandleResult<T> = Result<
    (
        jack::AsyncClient<SimpleNotificationHandler<T>, DummyProcessHandler>,
        Receiver<T>,
    ),
    jack::Error,
>;

/// `jack_notification_handle` creates a Jack Client that we need to listen to notifications.
///
/// it returns the active client and the receiver side of the channel for Notifications.
/// one need to hold the active client until one drops it to close the client.
///
/// # Errors
///
/// This function will return an error  if the async client cannot be created.
#[inline]
pub fn jack_notification_handle<T>(name: &str) -> JackHandleResult<T>
where
    T: From<Notification> + Send + 'static,
{
    let (client, _status) = Client::new(name, jack::ClientOptions::NO_START_SERVER)?;
    let (sender, receiver) = channel::<T>();

    let active_client = client.activate_async(
        SimpleNotificationHandler { msg_sender: sender },
        DummyProcessHandler {},
    )?;
    Ok((active_client, receiver))
}
