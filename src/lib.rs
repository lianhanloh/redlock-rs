//! redlock-rs is a Rust implementation of the Redlock algorithm for distributed lock management of
//! Redis nodes. 

extern crate redis;

// public api
pub use redlock::Redlock;

mod redlock;

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
