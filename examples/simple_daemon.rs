#[cfg(feature = "tokio")]
#[path="simple_daemon_tokio/mod.rs"]
mod simple_daemon_impl;
#[cfg(feature = "tokio-core")]
#[path="simple_daemon_tokio_core/mod.rs"]
mod simple_daemon_impl;

fn main() {
  simple_daemon_impl::main()
}
