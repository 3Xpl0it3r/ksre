pub(crate) mod collector;
pub(crate) mod client;
pub(crate) mod agent;
pub(crate) mod storage;
mod bytes;


pub use agent::SreAgent;


pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

