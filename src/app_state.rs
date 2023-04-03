use std::env::{self, VarError};

use reqwest::Url;

#[derive(Clone, Debug)]
pub enum DomainMatchMode {
    Whitelist,
    Blacklist,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub domains: Vec<Url>,
    pub mode: DomainMatchMode,
}

impl AppState {
    pub fn new() -> Result<AppState, VarError> {
        let mode = match env::var("MODE") {
            Ok(value) if value == "blacklist" => DomainMatchMode::Blacklist,
            _ => DomainMatchMode::Whitelist,
        };

        let domains = if env::var("DOMAINS").is_err() {
            Vec::new()
        } else {
            env::var("DOMAINS")?
                .split(";")
                .map(|x| Url::parse(x).unwrap())
                .collect()
        };

        Ok(AppState { domains, mode })
    }

    pub fn is_allowed(&self, url_string: &str) -> bool {
        let url = match Url::parse(url_string) {
            Ok(url) => url,
            Err(_) => return false,
        };
        let contains = self.domains
            .iter()
            .any(|domain| domain.domain() == url.domain());
        
        match &self.mode {
          DomainMatchMode::Whitelist => return contains,
          DomainMatchMode::Blacklist => return !contains,
        }
    }
}
