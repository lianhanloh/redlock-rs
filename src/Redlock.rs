//! This mod specifies and implements the Distributed Lock Manager Redlock object, as well 
//! as the necessary functions lock() and unlock(). It also declares the struct Lock which 
//! is used to hold necessary information about each lock a client acquires. 

use redis::{Client, Connection, RedisResult, Commands};
use types::{RedlockResult, Error};
use time::precise_time_s;
use std::time::Duration;
use std::thread::sleep;
use rand::thread_rng;

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
        "1234567".to_string();
    }

   /// locks resource specified by res_name for ttl in miliseconds
    pub fn lock(&mut self, res_name: String, ttl: i32) -> RedlockResult<Lock> {
        let mut retry = 0;
        let val = self.get_unique_id();
        let drift : i32 = (((ttl as f32) * self.clock_drift_factor) as i32) + 2;
        while retry < self.retry_count {
            let mut n = 0;
            let start_time : i32 = (precise_time_s() * 1000.0) as i32;
            for server in &mut self.servers {
                let res = lock_instance(server, &res_name, &val, ttl);
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
                for server in &mut self.servers {
                    let res = unlock_instance(server, &res_name, &val); 
                    if res.is_err() { 
                        return Err(Error::RedlockConn);
                    }
                }
                retry = retry + 1;
                // sleep for retry_delay
                sleep(Duration::from_millis((self.retry_delay as u64) * 1000));
            }
        }
        Err(Error::CannotObtainLock)
    }
    /// unlocks resource held by Lock
    pub fn unlock(&mut self, lock: Lock) -> RedlockResult<()> {
        for server in &mut self.servers {
            let res = unlock_instance(server, &lock.resource, &lock.key);
            if res.is_err() {
                return Err(Error::RedlockConn);
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

/// release lock from one server
fn unlock_instance(server : &Connection, res_name : &str, 
                   val : &str) -> RedlockResult<()> {
    let res : RedisResult<String> = server.get(res_name.to_string());
    if res.is_ok() {
        let v = res.unwrap();
        if val.to_string() == v {
            let res : RedisResult<String> = server.del(res_name.to_string());
            if res.is_ok() {
                Ok(())
            } else {
                //TODO: retry?
                Err(Error::RedlockConn)
            }
        } else {
            Err(Error::InvalidLock)
        }
    } else {
        Err(Error::RedlockConn)
    }
}

/// acquire lock from one server
fn lock_instance(server : &Connection, res_name : &str, val : &str, 
                 ttl: i32) -> RedlockResult<()> {
    let res : RedisResult<String> = server.set_nx(res_name.to_string(), val.to_string());
    if res.is_ok() {
        let res : RedisResult<String> = server.set_ex(res_name.to_string(),
        val.to_string(), ttl as usize);
        if res.is_ok() {
            Ok(())
        } else {
            Err(Error::RedlockConn)
        }
    } else {
        Err(Error::RedlockConn)
    }
}


