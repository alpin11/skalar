use reqwest::Client;
use reqwest::StatusCode;
use std::io::{self, Write};

const MAX_SIZE: u64 = 100 * 1024 * 1024; // Maximum response size in bytes (100 MB)

pub enum FetchError {
    TooLarge,
    SendError,
    ChunkError,
}

impl FetchError {
    pub fn to_string(&self) -> String {
        match &self {
            FetchError::ChunkError => "Error Decoding Response".to_string(),
            FetchError::SendError => "Error Sending Request".to_string(),
            FetchError::TooLarge => {
                format!("Response Body Exceeded the maximum of {} bytes", MAX_SIZE)
            }
        }
    }

    pub fn to_http_error(&self) -> (StatusCode, String) {
        match &self {
            FetchError::ChunkError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FetchError::SendError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FetchError::TooLarge => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
        }
    }
}

pub async fn fetch_data(url: &str) -> Result<Vec<u8>, FetchError> {
    let client = Client::new();
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|_| FetchError::SendError)?;

    let mut content_length = 0;
    let mut body = Vec::new();
    let mut writer = io::Cursor::new(&mut body);
    loop {
        // stream the response so we can check how large the requested data is
        // without having to download the entire thing
        let chunk = response.chunk().await;
        let chunk = chunk.map_err(|_| FetchError::ChunkError)?;
        if chunk.is_none() {
            break;
        }
        let chunk = chunk.unwrap();
        let chunk_size = chunk.len() as u64;
        content_length += chunk_size;
        if content_length > MAX_SIZE {
            return Err(FetchError::TooLarge);
        }
        writer
            .write_all(&chunk)
            .map_err(|_| FetchError::ChunkError)?;
    }

    Ok(body)
}
