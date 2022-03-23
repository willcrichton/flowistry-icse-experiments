//! A function for parsing URLs. See the doc comment below for details.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Url {
  pub scheme: Option<String>,
  pub hostname: String,
  pub tld: Option<String>,
  pub port: Option<usize>,
  pub path: Vec<String>,
  pub query: Option<String>,
}

/// Parses a URL string into its components. For example:
///
/// https://github.com:80/willcrichton/flowistry?nocache
///    ^      ^    ^   ^       ^           ^       ^
///    |      |    |   |       |           |       |
///    |      |   tld  |       |           |     query
/// scheme  hostname  port   path[0]     path[1]
///
/// Only handles these URL components, and not eg: subdomains, usernames, so on.
pub fn decode_url(url: &str) -> Option<Url> {
  let chars = &mut url.chars();

  let scheme = if url.contains("://") {
    // note: take_while also consumes the iterator element where the predicate is false,
    // so the first ':' will be removed from `chars`
    let scheme = chars.take_while(|c| *c != ':').collect::<String>();
    if chars.take(2).collect::<String>() != "//" {
      return None;
    }
    Some(scheme)
  } else {
    None
  };

  let domain_and_port = &mut chars.take_while(|c| *c != '/');
  let domain = domain_and_port
    .take_while(|c| *c != ':')
    .collect::<String>();
  if domain.len() == 0 {
    return None;
  }

  let mut domain_parts = domain.split('.');
  let hostname = domain_parts.next().unwrap().to_string();
  let tld = domain_parts.next().map(|s| s.to_string());

  let port_str = domain_and_port.collect::<String>();
  let port = if port_str.len() > 0 {
    Some(port_str.parse::<usize>().ok()?)
  } else {
    None
  };

  let mut path = Vec::new();
  loop {
    let part = chars.take_while(|c| *c != '/').collect::<String>();
    if part.len() > 0 {
      path.push(part);
    } else {
      break;
    }
  }

  let query = match path.last() {
    Some(part) if part.contains('?') => {
      let page = path.pop().unwrap();
      let (_, query) = page.split_once('?').unwrap();
      Some(query.to_string())
    }
    _ => None,
  };

  Some(Url {
    scheme,
    hostname,
    tld,
    port,
    path,
    query,
  })
}

#[test]
fn url_test1() {
  assert_eq!(
    Some(Url {
      scheme: Some("https".into()),
      hostname: "hey".into(),
      tld: None,
      port: Some(30),
      path: vec!["foo".into(), "bar".into()],
      query: None
    }),
    decode_url("https://hey:30/foo/bar")
  );
}

#[test]
fn url_test2() {
  assert_eq!(
    Some(Url {
      scheme: None,
      hostname: "test".into(),
      tld: Some("com".into()),
      port: None,
      path: vec!["ok".into()],
      query: None
    }),
    decode_url("test.com/ok")
  );
}

#[test]
fn url_test3() {
  assert_eq!(
    Some(Url {
      scheme: Some("http".into()),
      hostname: "example".into(),
      tld: Some("com".into()),
      port: Some(8080),
      path: vec!["yep".into()],
      query: Some("aquery".into()),
    }),
    decode_url("http://example.com:8080/yep?aquery")
  );
}
