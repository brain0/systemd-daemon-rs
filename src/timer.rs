use std::time::Duration;
use timerfd::*;
use std::{fmt, io};
use std::os::unix::io::AsRawFd;
use mio::{Evented, Poll, Token, Ready, PollOpt};
use mio::unix::EventedFd;

pub struct LinuxTimer(TimerFd);

impl LinuxTimer {
    pub fn new(tick: Duration) -> io::Result<LinuxTimer> {
        let mut timer = TimerFd::new_custom(ClockId::Monotonic, true, true)?;
        timer.set_state(TimerState::Periodic { current: tick, interval: tick }, SetTimeFlags::Default);
        Ok(LinuxTimer(timer))
    }

    pub fn read(&mut self) -> u64 {
        self.0.read()
    }
}

impl Evented for LinuxTimer {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.0.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.0.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.0.as_raw_fd()).deregister(poll)
    }
}

impl fmt::Debug for LinuxTimer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("LinuxTimer").field(&self.0.as_raw_fd()).finish()
    }
}
