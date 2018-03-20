#![cfg(target_os = "linux")]

#[cfg(not(any(feature = "tokio", feature = "tokio-core")))]
compile_error!("Either the feature \"tokio\" or \"tokio-core\" must be enabled for this crate.");
#[cfg(all(feature = "tokio", feature = "tokio-core"))]
compile_error!("The feature \"tokio\" and \"tokio-core\" are mutually exclusive.");

extern crate libsystemd;
extern crate timerfd;
extern crate futures;
extern crate mio;
#[cfg(feature = "tokio")]
extern crate tokio;
#[cfg(feature = "tokio-core")]
extern crate tokio_core;

mod error;
mod systemd_notifier;
mod timer;

pub use systemd_notifier::SystemdNotifier;
pub use error::Error;
