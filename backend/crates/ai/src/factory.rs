use std::sync::Arc;

use loglens_config::{AiProviderKind, AiSettings};

use crate::{AiProvider, OllamaProvider, OpenAiProvider};

/// Builds the provider selected by `LOGLENS_AI_PROVIDER`.
pub fn build_provider(settings: &AiSettings) -> Arc<dyn AiProvider> {
    match settings.provider {
        AiProviderKind::Ollama => Arc::new(OllamaProvider::new(&settings.url, &settings.model)),
        AiProviderKind::OpenAi => Arc::new(OpenAiProvider::new(
            &settings.url,
            &settings.model,
            settings.key.clone(),
        )),
    }
}
