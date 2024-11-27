use jack::Client;
use std::fmt::Display;
use std::sync::mpsc::{channel, Sender};

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

pub struct DummyProcessHandler {}

impl jack::ProcessHandler for DummyProcessHandler {
    fn process(&mut self, _c: &jack::Client, _ps: &jack::ProcessScope) -> jack::Control {
        jack::Control::Continue
    }

    fn buffer_size(&mut self, _c: &jack::Client, _size: jack::Frames) -> jack::Control {
        jack::Control::Continue
    }
}

pub struct SimpleNotificationHandler {
    pub sender: Sender<Notification>,
}

impl jack::NotificationHandler for SimpleNotificationHandler {
    fn thread_init(&self, _: &jack::Client) {
        let _ = self.sender.send(Notification::ThreadInit);
    }

    unsafe fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        let _ = self
            .sender
            .send(Notification::Shutdown(status, reason.to_owned()));
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        let _ = self.sender.send(Notification::Freewheel(is_enabled));
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        let _ = self.sender.send(Notification::SampleRate(srate));
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        let _ = self
            .sender
            .send(Notification::ClientRegistration(name.to_owned(), is_reg));
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        let _ = self
            .sender
            .send(Notification::PortRegistration(port_id, is_reg));
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        let _ = self.sender.send(Notification::PortRename(
            port_id,
            old_name.to_owned(),
            new_name.to_owned(),
        ));
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        let _ = self.sender.send(Notification::PortsConnected(
            port_id_a,
            port_id_b,
            are_connected,
        ));
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        let _ = self.sender.send(Notification::GraphReorder);

        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        let _ = self.sender.send(Notification::XRun);
        jack::Control::Continue
    }
}

pub fn jack_notification_handle(
    name: &str,
) -> Result<
    (
        jack::AsyncClient<SimpleNotificationHandler, DummyProcessHandler>,
        std::sync::mpsc::Receiver<Notification>,
    ),
    jack::Error,
> {
    let (client, _status) = Client::new(name, jack::ClientOptions::NO_START_SERVER)?;
    let (sender, receiver) = channel();

    let active_client = client
        .activate_async(SimpleNotificationHandler { sender }, DummyProcessHandler {})
        .unwrap();
    Ok((active_client, receiver))
}
