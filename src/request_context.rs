use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RequestContext {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub cache_max_age: Option<u32>,
}
