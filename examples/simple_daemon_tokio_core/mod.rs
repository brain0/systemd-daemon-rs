extern crate systemd_daemon;
extern crate tokio_core;

use self::systemd_daemon::SystemdNotifier;
use self::tokio_core::reactor::Core;

pub fn main() {
    println!("Creating the core ...");
    let mut core = Core::new().unwrap();
    println!("Creating the notifier ...");
    let mut notifier = SystemdNotifier::new(&core.handle());
    println!("Running the main loop ...");
    if let Err(err) = core.run(&mut notifier) {
        println!("Error: {}", err);
    }
    println!("Main loop exited");
}
