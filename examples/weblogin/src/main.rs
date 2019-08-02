extern crate gotham;
extern crate hyper;
extern crate mysteriouspants_esi;
extern crate serde;
extern crate serde_derive;
extern crate toml;

use gotham::helpers::http::response::create_permanent_redirect;
use gotham::router::builder::*;
use gotham::state::State;
use hyper::{Body, Response};
use mysteriouspants_esi::auth::web_login_url;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

#[derive(Debug)]
struct LoginResult {

}

#[derive(Deserialize, Debug)]
struct EsiSecrets {
  callback_url: String,
  client_id: String,
  secret_key: String,
  secret_salt: String,
  scopes: Vec<String>
}

const LOGIN_PROMPT: &str = r#"
 foo
"#;

fn login_prompt(state: State) -> (State, &'static str) {
  (state, LOGIN_PROMPT)
}

fn redirector(state: State) -> (State, Response<Body>) {
  // TODO: redirect to the EVE Online login url
  let url = web_login_url(
    "http://localhost:7878/callback_url", "client_id",
    &vec!["scope1", "scope2"], "secret state"
  );
  let resp = create_permanent_redirect(&state, url.to_string());

  (state, resp)
}

fn main() {
  // acquire esirs config
  let mut secrets_string = String::new();
     File::open(".secrets.toml")
       .expect("please place your dev secrets at .secrets.toml")
       .read_to_string(&mut secrets_string)
       .expect("could not read from secrets file");
  let secrets: EsiSecrets = toml::from_str(secrets_string.as_str())
    .expect("could not parse secrets file");

  println!("{:?}", secrets);

  let (tx, rx): (Sender<LoginResult>, Receiver<LoginResult>) = channel();

  thread::spawn(move || {
    let addr = "127.0.0.1:7878";
    println!("Waiting for you to log into EVE Online at http://{}", addr);

    gotham::start(addr, build_simple_router(|route| {
      route.get("/").to(login_prompt);
      route.get("/login").to(redirector);
    }));
  });

  let res = rx.recv();

  println!("{:?}", res);
}