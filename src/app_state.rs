use std::env::{self, VarError};
use regex::Regex;
use reqwest::Url;

#[derive(Clone, Debug)]
pub enum DomainMatchMode {
    Whitelist,
    Blacklist,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub domains: Vec<Regex>,
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
                .map(|s| s.replace("*", r".*"))
                .map(|s| Regex::new(&s).unwrap())
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
            .any(|domain| {
                if let Some(url) = url.host() {
                    let url = url.to_string();
                    domain.is_match(&url)
                } else {
                    false
                }
            });
        
        match &self.mode {
          DomainMatchMode::Whitelist => contains,
          DomainMatchMode::Blacklist => !contains,
        }
    }
}
