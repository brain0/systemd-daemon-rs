extern crate systemd_daemon;
extern crate tokio;
extern crate futures;

use self::systemd_daemon::SystemdNotifier;
use self::futures::prelude::*;

pub fn main() {
    println!("Creating the notifier ...");
    let notifier = SystemdNotifier::new();
    println!("Running the main loop ...");
    tokio::run(notifier.map_err(|err| {
        println!("Error: {}", err);
        ()
    }));
    println!("Main loop exited");
}
