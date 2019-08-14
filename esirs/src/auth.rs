use std::iter::Iterator;
use jsonwebtoken::{Algorithm, decode, TokenData, Validation};
use reqwest::Error as ReqwestError;
use serde::{Serialize, Deserialize};
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

pub enum Code2TokenError {
  ReqwestError(ReqwestError),
  ValidationError(jsonwebtoken::errors::Error),
  ExpiredTokenError(SystemTimeError)
}

pub fn code_to_token<T1: Into<String>>(
  code: T1
) -> Result<AuthToken, Code2TokenError> {
  // TODO: step the first: call https://login.eveonline.com/v2/oauth/token to
  //       exchange code for a JWT and refresh token

  // what is secret? is it something generated when i created the esi app?
  let claims = match decode::<EsiClaims>(
    code.into().as_str(), secret, &Validation::new(Algorithm::RS256)
  ) {
    Ok(claims) => claims,
    Err(err) => return Err(Code2TokenError::ValidationError(err))
  };

  // check that it isn't already expired
  let expiry = SystemTime::UNIX_EPOCH + Duration::from_secs(claims.claims.exp);

  match expiry.elapsed() {
    Ok(_) => (),
    Err(err) => return Err(Code2TokenError::ExpiredTokenError(err))
  };

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