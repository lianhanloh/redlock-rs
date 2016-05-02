//! This mod specifies and implements the Distributed Lock Manager Redlock object, as well 
//! as the necessary functions lock() and unlock(). It also declares the struct Lock which 
//! is used to hold necessary information about each lock a client acquires. 

use redis::{Client, Connection, RedisResult, cmd, Value};
use types::{RedlockResult, Error};
use time::precise_time_s;
use std::time::Duration;
use std::thread::sleep;
use rand::{thread_rng, Rng};

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
    /// start time in seconds returned by Rust's `precise_time_s()` from which `validity` runs from
    start_time: f64,
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
        let id_len = 22;
        thread_rng().gen_ascii_chars().take(id_len).collect()
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
            let start_time = precise_time_s();
            if validity > 0 && n >= self.quorum {
                // lock successful!
                return Ok(Lock::new(validity, res_name, val, start_time));
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
    pub fn new(validity: i32, res: String, key: String, start_time: f64) -> Self {
        Lock { validity: validity, resource: res, key: key, start_time: start_time, }
    }

    /// checks if lock is still valid
    pub fn still_valid(&self) -> bool {
        (((precise_time_s() - self.start_time) * 1000.0) as i32) < self.validity
    }
}

/// release lock from one server
fn unlock_instance(server : &Connection, res_name : &str, 
                   val : &str) -> RedlockResult<()> {
    let unlock_script = "if redis.call('get',KEYS[1]) == ARGV[1] then return redis.call('del',KEYS[1]) else return 0 end";
    let res : RedisResult<i32> = cmd("EVAL").arg(unlock_script).arg(1).arg(res_name).arg(val)
                                    .query(server);
    match res {
        Ok(1) => Ok(()),
        Ok(0) => Err(Error::UnlockFailed),
        _ => Err(Error::RedlockConn),
    }
}

/// acquire lock from one server
fn lock_instance(server : &Connection, res_name : &str, val : &str, 
                 ttl: i32) -> RedlockResult<()> {
    let res : RedisResult<Value> = cmd("SET").arg(res_name).arg(val).arg("NX").arg("PX")
                                    .arg(ttl).query(server);
    match res {
        Ok(Value::Okay) => Ok(()),
        Ok(Value::Nil) => Err(Error::CannotObtainLock),
        _ => Err(Error::RedlockConn),
    }
}

#[test]
pub fn test_lock_instance() {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    assert!(lock_instance(&con, "lock_test_res", "uni_val", 1000).is_ok());
    assert!(lock_instance(&con, "lock_test_res", "uni_val", 1000).is_err());
    sleep(Duration::from_secs(1));
    assert!(lock_instance(&con, "lock_test_res", "uni_val", 10).is_ok());
}

#[test]
pub fn test_unlock_instance() {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    assert!(lock_instance(&con, "unlock_test_res", "uni_val", 30000).is_ok());
    let res = unlock_instance(&con, "unlock_test_res", "uni_val");
    assert!(res.is_ok());
    assert_eq!(res, Ok(()));
    let res = unlock_instance(&con, "unlock_test_res", "uni_val");
    assert!(res.is_err());
}


#[cfg(test)]
mod test{
    use super::Redlock;
    use redis;
    use redis::{RedisResult, Value};

    #[test]
    pub fn redis_check() {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let con = client.get_connection().unwrap();
        let res : RedisResult<Value> = redis::cmd("SET").arg("res").arg("key").arg("NX").arg("PX").arg(3000).query(&con);
        assert_eq!(res, Ok(Value::Okay));
        assert_eq!(redis::cmd("GET").arg("res").query(&con), Ok("key".to_string()));
        let res : RedisResult<Value> = redis::cmd("SET").arg("res").arg("key").arg("NX").arg("PX").arg(3000).query(&con);
        assert_eq!(res, Ok(Value::Nil));
    }

    #[test]
    pub fn single_server_lock() {
        let mut dlm = Redlock::dlm(vec!["redis://127.0.0.1".to_string()], None, None).unwrap();
        let my_lock = dlm.lock("my_resource_name".to_string(), 5000);
        assert!(my_lock.is_ok());
        let lock_should_fail = dlm.lock("my_resource_name".to_string(), 4000);
        assert!(lock_should_fail.is_err());
    }

    /*
    #[test]
    pub fn missing_server() {
        assert!(Redlock::dlm(vec!["redis://123.123.123.123".to_string()], None, None).is_err());
    }
    */

    #[test]
    pub fn multi_server() {
        let urls = vec!["redis://127.0.0.1:6378".to_string(),
                        "redis://127.0.0.1:6379".to_string(),
                        "redis://127.0.0.1:6111".to_string()];

        let mut dlm = Redlock::dlm(urls, None, None).unwrap();
        let lock = dlm.lock("multi_lock".to_string(), 10000);
        assert!(lock.is_ok());
        let lock = dlm.lock("multi_lock".to_string(), 10000);
        assert!(lock.is_err());
    }
}
