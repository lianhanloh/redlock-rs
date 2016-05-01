//! redlock-rs is a Rust implementation of the Redlock algorithm for distributed lock management of
//! Redis nodes. 

extern crate redis;
extern crate time;
extern crate rand;

//TODO: check out docopt -> command line argument parser plugin

pub mod redlock;
pub mod types;
