//! redlock-rs is a Rust implementation of the Redlock algorithm for distributed lock management of
//! Redis nodes. 

extern crate redis;

//TODO: check out docopt -> command line argument parser plugin

// public api
pub use redlock::Redlock;

mod redlock;
mod types;

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
