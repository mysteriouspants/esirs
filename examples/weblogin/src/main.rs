extern crate gotham;
extern crate hyper;
extern crate mysteriouspants_esi;

use mysteriouspants_esi::auth::web_login_url;

use hyper::{Body, Response};

use gotham::helpers::http::response::create_permanent_redirect;
use gotham::router::builder::*;
use gotham::state::State;

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug)]
struct LoginResult {

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