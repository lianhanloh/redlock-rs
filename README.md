# redlock-rs
Redis distributed locks in Rust

This Rust crate implements the Redis-based distributed lock manager algorithm [described in this blog post](http://redis.io/topics/distlock).

Documentation on the library can be found at https://lianhanloh.github.io/redlock-rs.

To create a lock manager and acquire lock:

```rust
extern crate redlock;
use redlock::Redlock;

  let dlm = Redlock::dlm(vec!["redis://127.0.0.1".to_string()], None, None).unwrap();
  let my_lock = dlm.lock("my_resource_name".to_string(), 1000);

```
