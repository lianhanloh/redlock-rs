//! redlock-rs is a Rust implementation of the Redlock algorithm for distributed lock management of
//! Redis nodes. 

extern crate redis;

mod dlm;

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
