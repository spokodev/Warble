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
    previous_text: &str,
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

    // Build prompt: previous transcription context + vocabulary
    // Whisper uses ~224 token limit for prompt, ~4 chars per token = ~900 chars max
    if provider.supports_vocabulary {
        let mut prompt_parts = Vec::new();
        if !previous_text.is_empty() {
            // Truncate previous text to ~400 chars to leave room for vocabulary
            let prev = truncate_str(previous_text, 400);
            prompt_parts.push(prev.to_string());
        }
        if !vocabulary.is_empty() {
            prompt_parts.push(vocabulary.to_string());
        }
        let prompt = prompt_parts.join(". ");
        if !prompt.is_empty() {
            // Truncate total prompt to ~900 chars (~224 tokens)
            let truncated = truncate_str(&prompt, 900);
            form = form.text("prompt", truncated.to_string());
        }
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

/// Truncate a string to at most `max_bytes` without splitting a UTF-8 character.
fn truncate_str(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
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
