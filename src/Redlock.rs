//! This mod specifies and implements the Distributed Lock Manager Redlock object, as well 
//! as the necessary functions lock() and unlock(). It also declares the struct Lock which 
//! is used to hold necessary information about each lock a client acquires. 

use redis::{Client, Connection};
use types::{RedlockResult, Error};
use time::precise_time_s;

/// Distributed Lock Manager class object
pub struct Redlock {
    /// list of connections with Master Nodes
    servers: Vec<Connection>,
    /// no. of retries to secure locks
    retry_count: i32,
    /// time delay between each retry
    retry_delay: f32,
    /// number of locks needed
    quorum: i32,
    /// clock drift factor
    clock_drift_factor: f32,
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
        let quorum = urls.len() as i32;
        for u in urls {
            let client_res = Client::open(&*u);
            if client_res.is_ok() {
                let con_res = client_res.unwrap().get_connection();
                if con_res.is_ok() {
                    servers.push(con_res.unwrap());
                }
            }
        }
        if (servers.len() as i32) < quorum {
            return Err(Error::NotEnoughMasters);
        }
        let rc = match retry_count {
            Some(x) => x,
            None => 3,
        };
        let rd = match retry_delay {
            Some(x) => x,
            None => 0.2,
        };
        let cdf = 0.01;
        Ok(Redlock { 
            servers: servers, 
            retry_count: rc, 
            retry_delay: rd,
            quorum: quorum,
            clock_drift_factor: cdf,
        })
    }

    /// generates unique id for lock key
    fn get_unique_id(&self) -> String {
        unimplemented!()
    }

    /// acquire lock from one server
    fn lock_instance(&self, server : &Connection, res_name : &str, val : &str, 
                     ttl: i32) -> RedlockResult<()> {
        Ok(())
    }

    /// release lock from one server
    fn unlock_instance(&self, server : Connection, res_name : &str, 
                       val : &str) -> () {
    }

    /// locks resource specified by res_name for ttl in miliseconds
    pub fn lock(&self, res_name: String, ttl: i32) -> RedlockResult<Lock> {
        let mut retry = 0;
        let val = self.get_unique_id();
        let drift : i32 = (((ttl as f32) * self.clock_drift_factor) as i32) + 2;
        while retry < self.retry_count {
            let mut n = 0;
            let start_time : i32 = (precise_time_s() * 1000.0) as i32;
            for server in self.servers {
                let res = self.lock_instance(server, &res_name, &val, ttl);
                if res.is_ok() {
                    n = n + 1;
                }
            }
            let elapsed_time : i32 = ((precise_time_s() * 1000.0) as i32) - start_time;
            let validity = ttl - elapsed_time - drift;
            if validity > 0 && n >= self.quorum {
                // lock successful!
                return Ok(Lock::new(validity, res_name, val));
            }  else {
                for server in self.servers {
                    self.unlock_instance(server, &res_name, &val); 
                }
                retry = retry + 1;
                //TODO: sleep for retry_delay
            }
        }
        Err(Error::CannotObtainLock)
    }
    /// unlocks resource held by Lock
    pub fn unlock(lock: Lock) -> RedlockResult<()> {
        for server in self.servers {
            let res = self.unlock_instance(server, lock.resource, lock.key);
            if res.is_err() {
                Err(Error::MultipleRedlock)
            }
        }
        Ok(())
    }
}

impl Lock {
    /// instantialize a Lock with validity, resource name, and key
    pub fn new(validity: i32, res: String, key: String) -> Self {
        Lock { validity: validity, resource: res, key: key }
    }
}
