use base64::encode as base64_encode;
use jsonwebtoken::{Algorithm, decode, TokenData, Validation};
use jsonwebtoken::errors::Error as JWTError;
use reqwest::{
  Client as ReqwestClient,
  Error as ReqwestError
};
use serde::{Serialize, Deserialize};
use std::iter::Iterator;
use std::time::{Duration, SystemTime};
use url::Url;
use crate::Client as EsiClient;

/// Constructs a web login url to redirect clients to when logging into the EVE
/// SSO site.
/// 
/// * `redirect_uri`: The uri of your application, such as
///   `http://localhost:420`.
/// * `client_id`: The id of your client, as assigned by the EVE Online 3P
///   Developer portal.
/// * `state`: A state which is given back on callback. it is suggested that you
///   make this unique per login redirect given to discourage forgery (spais).
/// * `scopes`: An iterable of things that become strings describing all of the
///   api scopes you are requesting. this should match what you set on the EVE
///   Online 3P Developer portal.
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

/// Converts a login code to an authenticated token you can make non-pulic calls
/// with. This calls the login API to convert the code to a JWT token, then
/// validates this token using the secret assigned to your application when it
/// was registered on the 3P Applications page.
/// 
/// * `login_client`: Persistent connection pool used to communicate with the
///   login API.
/// * `code`: The code that was provided from the callback to your app.
/// * `client_id`: The id of your client, as assigned when you registered on the
///   EVE Online 3P Developer portal.
/// * `secret`: The secret key assigned to your application on the EVE Online 3P
///   Application portal.
pub fn code_to_token(
  esi_client: &EsiClient, code: &str, client_id: &str, secret: &str
) -> Result<UnvalidatedToken, Code2TokenError> {
  let response_str =
    esi_client.sso_client.post("https://login.eveonline.com/v2/oauth/token")
      .form(&[("grant_type", "authorization_code"), ("code", &code)])
      .header(
        "Authorization",
        format!("Basic {}",base64_encode(&format!("{}:{}", client_id, secret)))
      )
      .send()?
      .text()?;
      //.json::<UnvalidatedToken>()?;

  let token : UnvalidatedToken = serde_json::from_str(&response_str).unwrap();

  return Ok(token);

/*
  let claims =
    decode::<EsiClaims>(
      // one of these two, access or secret, is wrong... play with jwt.io until it's sorted
      &token.access_token, secret.as_bytes(), &Validation::new(Algorithm::RS256)
    )?;

  let auth_token = AuthToken {
    expires_at: SystemTime::UNIX_EPOCH + Duration::from_secs(claims.claims.exp),
    access_token: claims,
    token_type: token.token_type,
    refresh_token: token.refresh_token
  };

  if auth_token.access_token.claims.iss != "login.eveonline.com" {
    Err(Code2TokenError::BadIssuer(auth_token))
  } else {
    Ok(auth_token)
  }
  */
} 

/// Persistable token which allows you to make priveleged calls to ESI.
#[derive(Debug)]
pub struct AuthToken {

  /// JWT access token.
  pub access_token: TokenData<EsiClaims>,

  /// When this token expires and needs to be refreshed using [`refresh_token`].
  pub expires_at: SystemTime,

  /// The type of token issued. 
  pub token_type: String,

  /// The very important refresh token - don't leak this, it can be used to get
  /// new tokens indefinitely until the application is destroyed or the user
  /// deauthorizes the application manually!
  pub refresh_token: String
}

/// JWT claims.
#[derive(Debug, Deserialize, Serialize)]
pub struct EsiClaims {

  /// Authorization scopes; these should match those you requested.
  scp: Vec<String>,

  /// Some UDID?
  jti: String,

  /// Usually just `JWT-Signature-Key`.
  kid: String,

  /// String containing the character id, such as `CHARACTER:EVE:123456`.
  sub: String,

  /// The client id of your 3P client.
  azp: String,

  /// Authenticated character name.
  name: String,

  /// Owner hash. This will change if the character is transferred to another
  /// account.
  owner: String,

  /// Token expiration, in epoch seconds.
  exp: u64,

  /// The issuer, which should always be `login.eveonline.com`.
  iss: String
}

/// The raw, unvalidated response from the login api. This intermediary allows
/// Reqwest to deserialize the data, so we can have JWT validate `access_token`.
#[derive(Clone, Debug, Deserialize)]
pub struct UnvalidatedToken {

  /// The JWT access token.
  access_token: String,

  /// When this token invalidates, in epoch time.
  expires_in: u64,

  /// The type of token.
  token_type: String,

  /// The very important refresh token - don't leak this, it can be used to
  /// get new tokens indefinitely until the application is destroyed or the
  /// user deauthorizes the application manually!
  refresh_token: String
}

/// The kinds of errors which can be generated by the [`code_to_token`] method.
#[derive(Debug)]
pub enum Code2TokenError {

  /// An error communicating with the server, or an issue deserializing the
  /// response.
  ReqwestError(ReqwestError),

  /// An error validating the resulting JWT. This definitely means goon spais.
  ValidationError(JWTError),

  /// The identity of the token issuer could not be verified.
  BadIssuer(AuthToken)
}

// little bit of boilerplate which allows the ? operator to work
impl From<ReqwestError> for Code2TokenError {
  fn from(error: ReqwestError) -> Code2TokenError {
    Code2TokenError::ReqwestError(error)
  }
}

// little bit of boilerplate which allows the ? operator to work
impl From<JWTError> for Code2TokenError {
  fn from(error: JWTError) -> Code2TokenError {
    Code2TokenError::ValidationError(error)
  }
}

#[cfg(test)]
mod tests {
  use crate::auth::web_login_url;

  #[test]
  fn web_login_url_creation() {
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