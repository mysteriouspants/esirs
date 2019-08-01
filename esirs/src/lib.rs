extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;

pub mod auth;
pub mod search;

/// Base path to ESI.
pub const URL: &'static str = "https://esi.evetech.net/latest/";


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
