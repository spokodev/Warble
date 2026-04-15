use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProvider {
    pub name: String,
    pub url: String,
    pub default_model: String,
    pub response_format: String,
    pub auth_header: String,
    pub supports_vocabulary: bool,
    pub supports_model_choice: bool,
}

pub fn detect_provider(api_key: &str) -> Option<ApiProvider> {
    if api_key.starts_with("wsk-") {
        Some(ApiProvider {
            name: "Self-Hosted (spoko.dev)".to_string(),
            url: "https://whisper.spoko.dev/v1/audio/transcriptions".to_string(),
            default_model: "Systran/faster-whisper-small".to_string(),
            response_format: "verbose_json".to_string(),
            auth_header: "Bearer".to_string(),
            supports_vocabulary: true,
            supports_model_choice: true,
        })
    } else if api_key.starts_with("sk_") {
        Some(ApiProvider {
            name: "ElevenLabs".to_string(),
            url: "https://api.elevenlabs.io/v1/speech-to-text".to_string(),
            default_model: "scribe_v1".to_string(),
            response_format: "json".to_string(),
            auth_header: "xi-api-key".to_string(),
            supports_vocabulary: false,
            supports_model_choice: false,
        })
    } else if api_key.starts_with("gsk") {
        Some(ApiProvider {
            name: "Groq".to_string(),
            url: "https://api.groq.com/openai/v1/audio/transcriptions".to_string(),
            default_model: "whisper-large-v3".to_string(),
            response_format: "verbose_json".to_string(),
            auth_header: "Bearer".to_string(),
            supports_vocabulary: false,
            supports_model_choice: false,
        })
    } else if api_key.starts_with("sk-") {
        Some(ApiProvider {
            name: "OpenAI".to_string(),
            url: "https://api.openai.com/v1/audio/transcriptions".to_string(),
            default_model: "whisper-1".to_string(),
            response_format: "verbose_json".to_string(),
            auth_header: "Bearer".to_string(),
            supports_vocabulary: false,
            supports_model_choice: false,
        })
    } else {
        None
    }
}
