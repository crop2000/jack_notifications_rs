# jack_notification_rs
jack notifications for rust.
The jack crate exposes the callbacks to rust that make it possible to listen to jack notifications.
This library implements a `enum` for the possible notifications.
A simple jack client that is listens to those notifications and send them through a channel.
An example application prints those `Notifications` using the `Display` trait to `stdout`.