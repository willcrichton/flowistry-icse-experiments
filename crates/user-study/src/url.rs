#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Url {
  pub scheme: Option<String>,
  pub hostname: String,
  pub tld: Option<String>,
  pub port: Option<usize>,
  pub path: Vec<String>,
  pub query: Option<String>,
}

pub fn decode_url(url: &str) -> Option<Url> {
  let chars = &mut url.chars();

  let scheme = if url.contains("://") {
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

  let mut domain_parts = domain.split(".");
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

  let query = if path.last().map(|part| part.contains("?")).unwrap_or(false) {
    let page = path.pop().unwrap();
    let mut page_parts = page.split("?");
    page_parts.next().map(|query| query.to_string())
  } else {
    None
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
