use std::env::{self, VarError};

use reqwest::Url;

#[derive(Clone, Debug)]
pub enum UrlMethod {
    Whitelist,
    Blacklist,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub domains: Vec<Url>,
    pub method: UrlMethod,
}

impl AppState {
    pub fn new() -> Result<AppState, VarError> {
        let method = match env::var("METHOD") {
            Ok(value) if value == "blacklist" => UrlMethod::Blacklist,
            _ => UrlMethod::Whitelist,
        };

        let domains = if env::var("DOMAINS").is_err() {
            Vec::new()
        } else {
            env::var("DOMAINS")?
                .split(";")
                .map(|x| Url::parse(x).unwrap())
                .collect()
        };

        Ok(AppState { domains, method })
    }

    pub fn is_allowed(&self, url_string: &str) -> bool {
        let url = match Url::parse(url_string) {
            Ok(url) => url,
            Err(_) => return false,
        };
        let contains = self.domains
            .iter()
            .any(|domain| domain.domain() == url.domain());
        
        match &self.method {
          UrlMethod::Whitelist => return contains,
          UrlMethod::Blacklist => return !contains,
        }
    }
}
