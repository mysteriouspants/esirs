use base64::encode as base64_encode;
use jsonwebtoken::{Algorithm, decode, TokenData, Validation};
use reqwest::{
  Client as ReqwestClient,
  Error as ReqwestError,
  Result as ReqwestResult
};
use serde::{Serialize, Deserialize};
use std::iter::Iterator;
use std::time::{Duration, SystemTime, SystemTimeError};
use url::Url;

pub fn web_login_url<T1, T2>(
  redirect_uri: &str, client_id: &str, scopes: T1, state: &str
) -> Url where T1 : IntoIterator<Item = T2>, T2 : AsRef<str> {
  let joined_scopes = scopes.into_iter()
          .map(|t1i| String::from(t1i.as_ref()) )
          .collect::<Vec<_>>()
          .join(" ");
  Url::parse_with_params(
    "https://login.eveonline.com/v2/oauth/authorize/?response_type=code",
    &[
      ("redirect_uri", redirect_uri), ("client_id", client_id),
      ("scopes", &joined_scopes), ("state", state)
    ]
  ).expect("Could not construct sso login url")
}

#[derive(Serialize, Deserialize)]
pub struct EsiClaims {
  scp: Vec<String>,
  jti: String,
  kid: String,
  sub: String,
  azp: String,
  name: String,
  owner: String,
  /// Token expiration, in epoch seconds.
  exp: u64,
  iss: String
}

pub struct AuthToken {
  access_token: TokenData<EsiClaims>,
  expires_at: SystemTime,
  token_type: String,
  refresh_token: String
}

#[derive(Deserialize)]
struct UnvalidatedToken {
  access_token: String,
  expires_at: u64,
  token_type: String,
  refresh_token: String
}

pub enum Code2TokenError {
  ReqwestError(ReqwestError),
  ValidationError(jsonwebtoken::errors::Error)
}

/// secret = the secret key assigned when the EVE 3P Application was generated
pub fn code_to_token(
  login_client: &ReqwestClient, code: &str, client_id: &str, secret: &str
) -> Result<AuthToken, Code2TokenError> {
  let token = {
    let response = login_client.post("https://login.eveonline.com/v2/oauth/token")
      .form(&[("grant_type", "authorization_code"), ("code", &code)])
      .header(
        "Authorization",
        format!("Basic {}",base64_encode(&format!("{}:{}", client_id, secret)))
      )
      .send();

    let mut raw_response = match response {
      Ok(raw) => raw,
      Err(err) => return Err(Code2TokenError::ReqwestError(err))
    };

    let token: UnvalidatedToken = match raw_response.json::<UnvalidatedToken>() {
      Ok(token) => token,
      Err(err) => return Err(Code2TokenError::ReqwestError(err))
    };

    token
  };

  let claims = {
    let validation_result = decode::<EsiClaims>(
      &token.access_token, secret.as_bytes(), &Validation::new(Algorithm::RS256)
    );

    match validation_result {
      Ok(claims) => claims,
      Err(err) => return Err(Code2TokenError::ValidationError(err))
    }
  };

  let auth_token = AuthToken {
    expires_at: SystemTime::UNIX_EPOCH + Duration::from_secs(claims.claims.exp),
    access_token: claims,
    token_type: token.token_type,
    refresh_token: token.refresh_token
  };

  Ok(auth_token)
} 

#[cfg(test)]
mod tests {
  use crate::auth::web_login_url;

  #[test]
  fn it_works() {
    let url = web_login_url(
      "http://localhost/sso_callback", "my_client_id",
      &vec!["scope1", "scope2"], "secret_state_key"
    );

    assert!(
      url.as_str()
      == "https://login.eveonline.com/v2/oauth/authorize/?response_type=code&\
          redirect_uri=http%3A%2F%2Flocalhost%2Fsso_callback&client_id=\
          my_client_id&scopes=scope1+scope2&state=secret_state_key"
    );
  }
}