use url::Url;

pub fn web_login_url(
  redirect_uri: &str, client_id: &str, scopes: &Vec<&str>, state: &str
) -> Url {
  Url::parse_with_params(
    "https://login.eveonline.com/v2/oauth/authorize/?response_type=code",
    &[
      ("redirect_uri", redirect_uri), ("client_id", client_id),
      ("scopes", &scopes.join(" ")), ("state", state)
    ]
  ).expect("Could not construct sso login url")
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