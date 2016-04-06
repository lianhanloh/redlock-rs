use redis::Connection;
use std::result;

/// Distributed Lock Manager class object
pub struct Redlock {
    servers: Vec<Connection>,
}

/// Represents the Lock a client holds 
pub struct Lock {
    validity: i32,
    resource: String,
    key: String,
}

impl Redlock {
    pub fn new(urls : Vec<String>) -> Self {
        unimplemented!()
    }
    //TODO: change return type to a result
    /// locks resource specified by res_name for time in miliseconds
    pub fn lock(res_name: String, time: i32) -> Lock {
        unimplemented!()
    }
    /// unlocks resource held by Lock
    pub fn unlock(lock: Lock) -> () {
        unimplemented!()
    }
}
