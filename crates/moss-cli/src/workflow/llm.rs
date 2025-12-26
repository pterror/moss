//! LLM strategy for workflow execution.
//!
//! This module is only compiled when the "llm" feature is enabled.
//! Supports multiple providers: anthropic, openai, google, cohere, perplexity, xai.

#[cfg(feature = "llm")]
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers,
};

/// LLM strategy trait for workflow execution.
pub trait LlmStrategy: Send + Sync {
    /// Generate a completion from a prompt.
    fn complete(&self, prompt: &str) -> Result<String, String>;

    /// Generate with system prompt.
    fn complete_with_system(&self, system: &str, prompt: &str) -> Result<String, String>;
}

/// No LLM - for workflows that don't need it.
pub struct NoLlm;

impl LlmStrategy for NoLlm {
    fn complete(&self, _prompt: &str) -> Result<String, String> {
        Err("LLM not configured for this workflow".to_string())
    }

    fn complete_with_system(&self, _system: &str, _prompt: &str) -> Result<String, String> {
        Err("LLM not configured for this workflow".to_string())
    }
}

/// Supported LLM providers.
#[cfg(feature = "llm")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Google,
}

#[cfg(feature = "llm")]
impl Provider {
    /// Parse provider from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "anthropic" | "claude" => Some(Self::Anthropic),
            "openai" | "gpt" => Some(Self::OpenAI),
            "google" | "gemini" => Some(Self::Google),
            _ => None,
        }
    }

    /// Get default model for this provider.
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Anthropic => "claude-sonnet-4-20250514",
            Self::OpenAI => "gpt-4o",
            Self::Google => "gemini-2.0-flash",
        }
    }

    /// Get environment variable name for API key.
    pub fn env_var(&self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Google => "GEMINI_API_KEY",
        }
    }
}

#[cfg(feature = "llm")]
pub struct RigLlm {
    provider: Provider,
    model: String,
}

#[cfg(feature = "llm")]
impl RigLlm {
    pub fn new(provider_str: &str, model: Option<&str>) -> Result<Self, String> {
        let provider = Provider::from_str(provider_str)
            .ok_or_else(|| format!("Unsupported provider: {}", provider_str))?;

        // Check for API key
        if std::env::var(provider.env_var()).is_err() {
            return Err(format!(
                "Missing {} environment variable for {} provider",
                provider.env_var(),
                provider_str
            ));
        }

        let model = model
            .map(|m| m.to_string())
            .unwrap_or_else(|| provider.default_model().to_string());

        Ok(Self { provider, model })
    }

    async fn complete_async(&self, system: Option<&str>, prompt: &str) -> Result<String, String> {
        match self.provider {
            Provider::Anthropic => {
                let client = providers::anthropic::Client::from_env();
                let mut builder = client.agent(&self.model);
                if let Some(sys) = system {
                    builder = builder.preamble(sys);
                }
                let agent = builder.build();
                agent
                    .prompt(prompt)
                    .await
                    .map_err(|e| format!("Anthropic request failed: {}", e))
            }
            Provider::OpenAI => {
                let client = providers::openai::Client::from_env();
                let mut builder = client.agent(&self.model);
                if let Some(sys) = system {
                    builder = builder.preamble(sys);
                }
                let agent = builder.build();
                agent
                    .prompt(prompt)
                    .await
                    .map_err(|e| format!("OpenAI request failed: {}", e))
            }
            Provider::Google => {
                let client = providers::gemini::Client::from_env();
                let mut builder = client.agent(&self.model);
                if let Some(sys) = system {
                    builder = builder.preamble(sys);
                }
                let agent = builder.build();
                agent
                    .prompt(prompt)
                    .await
                    .map_err(|e| format!("Google request failed: {}", e))
            }
        }
    }
}

#[cfg(feature = "llm")]
impl LlmStrategy for RigLlm {
    fn complete(&self, prompt: &str) -> Result<String, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        rt.block_on(self.complete_async(None, prompt))
    }

    fn complete_with_system(&self, system: &str, prompt: &str) -> Result<String, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        rt.block_on(self.complete_async(Some(system), prompt))
    }
}

/// Build an LLM strategy from workflow config.
pub fn build_llm_strategy(_provider: Option<&str>, _model: Option<&str>) -> Box<dyn LlmStrategy> {
    #[cfg(feature = "llm")]
    {
        if let Some(provider) = _provider {
            match RigLlm::new(provider, _model) {
                Ok(llm) => return Box::new(llm),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize LLM: {}", e);
                }
            }
        }
    }

    Box::new(NoLlm)
}

/// List available providers.
#[cfg(feature = "llm")]
pub fn list_providers() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("anthropic", "claude-sonnet-4-20250514", "ANTHROPIC_API_KEY"),
        ("openai", "gpt-4o", "OPENAI_API_KEY"),
        ("google", "gemini-2.0-flash", "GEMINI_API_KEY"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_llm() {
        let llm = NoLlm;
        assert!(llm.complete("test").is_err());
    }

    #[test]
    fn test_build_llm_strategy_without_provider() {
        let strategy = build_llm_strategy(None, None);
        assert!(strategy.complete("test").is_err());
    }

    #[cfg(feature = "llm")]
    #[test]
    fn test_provider_parsing() {
        assert_eq!(Provider::from_str("anthropic"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_str("claude"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_str("openai"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_str("gpt"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_str("google"), Some(Provider::Google));
        assert_eq!(Provider::from_str("gemini"), Some(Provider::Google));
        assert_eq!(Provider::from_str("unknown"), None);
    }
}
