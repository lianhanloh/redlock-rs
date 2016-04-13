use redis::{Client, Connection};
use types::{RedlockResult, Error};


/// Distributed Lock Manager class object
pub struct Redlock {
    /// list of connections with Master Nodes
    servers: Vec<Connection>,
    /// no. of retries to secure locks
    retry_count: i32,
    /// time delay between each retry
    retry_delay: f32,
}

/// Represents the Lock a client holds 
pub struct Lock {
    /// an integer representing the number of milliseconds the lock will be valid
    validity: i32,
    /// the name of the locked resource as specified by the user.
    resource: String,
    /// a random value which is used to safely reclaim the lock.
    key: String,
}

impl Redlock {
    /// instantialize a lock manager with a vector of URLs (format: redis://host:port/db) 
    /// for each Redis master node, and optionally specify a retry count and delay between
    /// each retry. If None is given, the default will be 3 and 0.2s respectively
    pub fn dlm(urls : Vec<String>, retry_count: Option<i32>, retry_delay: Option<f32>)
        -> RedlockResult<Redlock> {
        let mut servers = Vec::new();
        for u in urls {
            let client_res = Client::open(&*u);
            if client_res.is_err() {
                return Err(Error::MultipleRedlock);
            }
            let con_res = client_res.unwrap().get_connection();
            if con_res.is_err() {
                return Err(Error::MultipleRedlock);
            }
            servers.push(con_res.unwrap());
        }
        let rc = match retry_count {
            Some(x) => x,
            None => 3,
        };
        let rd = match retry_delay {
            Some(x) => x,
            None => 0.2,
        };
        Ok(Redlock { 
            servers: servers, 
            retry_count: rc, 
            retry_delay: rd,
        })
    }
    /// locks resource specified by res_name for time in miliseconds
    pub fn lock(res_name: String, time: i32) -> RedlockResult<Lock> {
        unimplemented!()
    }
    /// unlocks resource held by Lock
    pub fn unlock(lock: Lock) -> RedlockResult<()> {
        unimplemented!()
    }

}
