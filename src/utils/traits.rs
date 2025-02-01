use std::thread::JoinHandle;

/// Specifies if a thread must be spawned
pub trait Runnable {
    fn run(self) -> Option<JoinHandle<()>>;
}

/// Specifies if it's a service
pub(crate) trait Service {
    fn start(self);
}
