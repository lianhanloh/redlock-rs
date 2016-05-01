# redlock-rs
Redis distributed locks in Rust

This Rust crate implements the Redis-based distributed lock manager algorithm [described in this blog post](http://redis.io/topics/distlock).

Documentation on the library can be found at https://lianhanloh.github.io/redlock-rs.

To create a lock manager:

```rust
extern crate redlock;
use redlock::Redlock;

  let dlm = Redlock::dlm(vec!["redis://127.0.0.1".to_string()], None, None).unwrap();
```

where the input arguments are a vector of Redis server URLs, as well as optional specifications for retry count and retry delay (in seconds).

To acquire a lock:

```rust
let my_lock = dlm.lock("my_resource_name".to_string(), 1000);
```

where the resource name is an unique identifier of what you are trying to lock and 1000 is the number of milliseconds you want the lock to be valid for. 

The returned value is a RedisResult<Lock>, with an Error::CannotObtainLock if a lock was not obtained. Otherwise, the struct Lock is returned, with its fields containing the resource name, the key used to safely release the lock, as well as a time for which it is valid for and the time from which it was acquired.

To release a lock:

```rust
dlm.unlock(my_lock);
```
which returns a RedlockResult<()>, Ok(()) if the associated resource was successfully unlocked and an Error otherwise

**Disclaimer**: This code implements an algorithm which is currently a proposal, it was not formally analyzed. Make sure to understand how it works before using it in your production environments.

Further Reading:
http://redis.io/topics/distlock
http://martin.kleppmann.com/2016/02/08/how-to-do-distributed-locking.html
http://antirez.com/news/101
https://medium.com/@talentdeficit/redlock-unsafe-at-any-time-40ceac109dbb#.uj9ffh96x
