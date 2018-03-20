use std::time::Duration;
use libsystemd::daemon::{notify, NotifyState, watchdog_enabled};
use futures::prelude::*;
#[cfg(feature = "tokio-core")]
use tokio_core::reactor::{PollEvented, Handle};
#[cfg(feature = "tokio")]
use tokio::reactor::{PollEvented2, Handle};
#[cfg(feature = "tokio")]
use mio::Ready;
use std::{io, mem};
use super::timer::LinuxTimer;
use super::error::Error;

#[cfg(feature = "tokio-core")]
#[derive(Debug)]
struct Timer(PollEvented<LinuxTimer>);

#[cfg(feature = "tokio-core")]
impl Timer {
    fn start(tick: Duration, reactor_data: &ReactorData) -> io::Result<Timer> {
        Ok(Timer(PollEvented::new(LinuxTimer::new(tick)?, &reactor_data.0)?))
    }

    fn poll(&mut self) -> io::Result<bool> {
        match self.0.poll_read() {
            Async::Ready(()) => {
                let ret = self.0.get_mut().read() > 0;
                self.0.need_read();
                Ok(ret)
            },
            Async::NotReady => Ok(false),
        }
    }
}

#[cfg(feature = "tokio-core")]
#[derive(Debug)]
struct ReactorData(Handle);

#[cfg(feature = "tokio")]
#[derive(Debug)]
struct Timer(PollEvented2<LinuxTimer>);

#[cfg(feature = "tokio")]
impl Timer {
    fn start(tick: Duration, reactor_data: &ReactorData) -> io::Result<Timer> {
        let timer = LinuxTimer::new(tick)?;
        match reactor_data.0 {
            Some(ref handle) => Ok(Timer(PollEvented2::new_with_handle(timer, handle)?)),
            None => Ok(Timer(PollEvented2::new(timer))),
        }
    }

    fn poll(&mut self) -> io::Result<bool> {
        match self.0.poll_read_ready(Ready::readable()) {
            Ok(Async::Ready(ready)) => {
                if ready.is_readable() {
                    let ret = self.0.get_mut().read() > 0;
                    self.0.clear_read_ready(Ready::readable())?;
                    Ok(ret)
                } else {
                    Ok(false)
                }
            },
            Ok(Async::NotReady) => Ok(false),
            Err(err) => Err(err),
        }
    }
}

#[cfg(feature = "tokio")]
#[derive(Debug)]
struct ReactorData(Option<Handle>);

#[derive(Debug)]
enum SystemdNotifierInner {
    Starting { reactor_data: ReactorData },
    Running { watchdog_timer: Timer },
}

/// A future for notifying systemd about daemon startup and pinging the watchdog.
///
/// When first polled, this future will notify systemd that daemon startup has completed.
/// If the systemd watchdog is disabled for this service, the future will then complete
/// successfully. Otherwise, it will ping the watchdog until the main loop shuts down.
///
/// If the application was not started with systemd or notify access was not enabled,
/// this future will complete with [`Error::NotRunningWithSystemd`](enum.Error.html#variant.NotRunningWithSystemd).
/// To ensure that the daemon works properly with or without systemd, this error should be silently ignored.
#[derive(Debug)]
pub struct SystemdNotifier(SystemdNotifierInner);

impl SystemdNotifier {
    /// Creates a new SystemdNotifier.
    #[cfg(feature = "tokio-core")]
    pub fn new(handle: &Handle) -> SystemdNotifier {
        SystemdNotifier(SystemdNotifierInner::Starting {
            reactor_data: ReactorData(handle.clone()),
        })
    }

    /// Creates a new SystemdNotifier using the default reactor.
    #[cfg(feature = "tokio")]
    pub fn new() -> SystemdNotifier {
        SystemdNotifier(SystemdNotifierInner::Starting {
            reactor_data: ReactorData(None),
        })
    }

    /// Creates a new SystemdNotifier.
    #[cfg(feature = "tokio")]
    pub fn new_with_handle(handle: &Handle) -> SystemdNotifier {
        SystemdNotifier(SystemdNotifierInner::Starting {
            reactor_data: ReactorData(Some(handle.clone())),
        })
    }

    fn notify_ready() -> bool {
        notify(false, &[NotifyState::Ready]).unwrap_or(false)
    }

    fn ping_watchdog(timer: &mut Timer) -> io::Result<()> {
        if timer.poll()? {
            let _ = notify(false, &[NotifyState::Watchdog]);
        }
        Ok(())
    }
}

impl Future for SystemdNotifier {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        let mut watchdog_timer = {
            let (watchdog_tick, reactor_data) = match self.0 {
                SystemdNotifierInner::Starting{ ref reactor_data } => {
                    if Self::notify_ready() {
                        if let Some(watchdog_tick) = watchdog_enabled(false) {
                            (watchdog_tick, reactor_data)
                        } else {
                            // Watchdog timer is not enabled
                            return Ok(Async::Ready(()));
                        }
                    } else {
                        // We are not running with systemd, or our service
                        // has NotifyAccess disabled.
                        return Err(Error::NotRunningWithSystemd)
                    }
                },
                SystemdNotifierInner::Running { ref mut watchdog_timer } => {
                    Self::ping_watchdog(watchdog_timer)?;
                    return Ok(Async::NotReady);
                },
            };

            Timer::start(watchdog_tick / 2, &reactor_data)?
        };
        Self::ping_watchdog(&mut watchdog_timer)?;
        mem::replace(&mut self.0, SystemdNotifierInner::Running { watchdog_timer });
        Ok(Async::NotReady)
    }
}
