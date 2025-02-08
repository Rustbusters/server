use std::thread::JoinHandle;

/// Specifies if a thread must be spawned
pub trait Runnable {
    fn run(self) -> Option<JoinHandle<()>>;
}

/// Specifies a running service such as a WebSocket or HTTP server.
pub(crate) trait Service {
    fn start(self);
}
