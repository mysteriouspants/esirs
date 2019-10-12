extern crate base64;
extern crate reqwest;
extern crate serde_json;
extern crate serde;
extern crate url;

pub mod auth;
pub mod search;

/// Base path to ESI.
pub const URL: &'static str = "https://esi.evetech.net/latest/";

#[derive(Clone, Debug)]
pub struct Client {
    sso_client: reqwest::Client
}

impl Client {
    pub fn new() -> Client {
        Client {
            sso_client: reqwest::Client::new()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
