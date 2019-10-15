extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate mysteriouspants_esi;
extern crate serde_derive;
extern crate serde;
extern crate toml;

use gotham::helpers::http::response::create_permanent_redirect;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single_middleware;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use hyper::{Body, Response};
use mysteriouspants_esi::auth::{web_login_url, code_to_token, UnvalidatedToken};
use mysteriouspants_esi::Client as EsiClient;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

#[derive(Deserialize, StateData, StaticResponseExtender, Debug)]
struct CallbackQueryStringExtractor {
  code: String,
  state: String
}

#[derive(Clone, Deserialize, Debug, StateData)]
struct EsiSecrets {
  callback_url: String,
  client_id: String,
  secret_key: String,
  secret_salt: String,
  scopes: Vec<String>
}

#[derive(Clone, Debug, StateData)]
struct ServiceState {
  esi_secrets: EsiSecrets,
  response_channel: Arc<Mutex<Sender<UnvalidatedToken>>>,
  esi_client: Arc<Mutex<EsiClient>>
}

fn html_response<T: Into<String>>(content: T) -> (mime::Mime, String) {
  (mime::TEXT_HTML, content.into())
}

fn login_prompt(state: State) -> (State, (mime::Mime, String)) {
  (
    state,
    html_response(
      r#"
        <?doctype html>
        <html>
          <head>
            <title>Log into EVE Online</title>
          </head>
          <body>
            <a href="/login">Log into EVE Online</a>
          </body>
        </html>
      "#
    )
  )
}

fn redirector(state: State) -> (State, Response<Body>) {
  let secrets = &ServiceState::borrow_from(&state).esi_secrets;
  let redirect_url = 
    web_login_url(
      secrets.callback_url.as_str(), secrets.client_id.as_str(),
      &secrets.scopes, secrets.secret_salt.as_str()
    ).to_string();
  let resp = create_permanent_redirect(&state, redirect_url);

  (state, resp)
}

fn callback_handler(mut state: State) -> (State, (mime::Mime, String)) {
  let query_params = CallbackQueryStringExtractor::take_from(&mut state);
  let service_state = &ServiceState::borrow_from(&state);


  let expected = &service_state.esi_secrets.secret_salt;
  let actual = &query_params.state;

  if expected != actual {
    // unauthorized response
    (
      state,
      html_response(
        r#"
          <?doctype html>
          <html>
            <head>
              <title>Authorisation failed</title>
            </head>
            <body>
              <strong>Authorisation failed.</strong>
              <p/>
              Unable to prove the callback came from ESI.
            </body>
          </html>
        "#
      )
    )
  } else {
    // authorized response
    let auth_token = {
      let esi_client = &service_state.esi_client.lock().expect("Cannot acquire esi client!");
      code_to_token(
        &esi_client, &query_params.code, &service_state.esi_secrets.client_id,
        &service_state.esi_secrets.secret_key
      ).expect("Failed to auth with the SSO server")
    };

    {
      let response_channel = service_state.response_channel.lock().expect("Cannot aquire response channel");
      response_channel.send(auth_token.clone()).expect("Could not send over channel");
    }

    (
      state,
      html_response(
        format!(r#"
          <?doctype html>
          <html>
            <head>
              <title>Authorisation successful</title>
            </head>
            <body>
              <strong>Authorisation successful</strong>
              <p/>
              auth_token: {:?}
            </body>
          </html>
        "#, &auth_token)
      )
    )
  }
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
  let (tx, rx): (Sender<UnvalidatedToken>, Receiver<UnvalidatedToken>) = channel();
  let service_state = ServiceState {
    esi_secrets: secrets,
    response_channel: Arc::new(Mutex::new(tx)),
    esi_client: Arc::new(Mutex::new(EsiClient::new()))
  };

  let state_data_middleware = StateMiddleware::new(service_state);
  let pipeline = single_middleware(state_data_middleware);
  let (chain, pipelines) = single_pipeline(pipeline);

  thread::spawn(move || {
    let addr = "127.0.0.1:7878";
    println!("Waiting for you to log into EVE Online at http://{}", addr);

    gotham::start(addr, build_router(chain, pipelines, |route| {
      route.get("/").to(login_prompt);
      route.get("/login").to(redirector);
      route.get("/callback_url")
           .with_query_string_extractor::<CallbackQueryStringExtractor>()
           .to(callback_handler);
    }));
  });

  let res = rx.recv();

  println!("{:?}", res);
}