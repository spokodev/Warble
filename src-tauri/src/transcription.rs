use crate::providers::{detect_provider, ApiProvider};
use reqwest::multipart;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
}

pub async fn transcribe(
    audio_path: &Path,
    api_key: &str,
    language: &str,
    vocabulary: &str,
    model_override: &str,
) -> Result<String, String> {
    let provider = detect_provider(api_key)
        .ok_or_else(|| "Unknown API key format. Supported: wsk-, sk_, gsk, sk-".to_string())?;

    // Use user-selected model if provider supports it, otherwise provider default
    let model = if provider.supports_model_choice && !model_override.is_empty() {
        model_override.to_string()
    } else {
        provider.default_model.clone()
    };

    let audio_data = tokio::fs::read(audio_path)
        .await
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    let file_part = multipart::Part::bytes(audio_data)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to set MIME type: {}", e))?;

    let mut form = multipart::Form::new()
        .part("file", file_part)
        .text("model", model)
        .text("response_format", provider.response_format.clone());

    if !language.is_empty() {
        form = form.text("language", language.to_string());
    }

    if provider.supports_vocabulary && !vocabulary.is_empty() {
        form = form.text("prompt", vocabulary.to_string());
    }

    let client = reqwest::Client::new();
    let mut request = client.post(&provider.url).multipart(form);

    request = add_auth_header(request, &provider, api_key);

    let response = request
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }

    let result: TranscriptionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let text = result.text.trim().to_string();
    if text.is_empty() {
        return Err("Transcription returned empty text".to_string());
    }

    Ok(text)
}

fn add_auth_header(
    request: reqwest::RequestBuilder,
    provider: &ApiProvider,
    api_key: &str,
) -> reqwest::RequestBuilder {
    if provider.auth_header == "xi-api-key" {
        request.header("xi-api-key", api_key)
    } else {
        request.bearer_auth(api_key)
    }
}
