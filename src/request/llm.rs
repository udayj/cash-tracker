use crate::database::DatabaseService;
use crate::configuration::Context;
use crate::core::RetryableClient;
use std::env;
use std::fs;
use std::sync::Arc;
use thiserror::Error;
use serde_json::json;

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("Missing GROQ_API_KEY in environment")]
    MissingApiKey,

    #[error("Failed to read system prompt: {0}")]
    SystemPromptReadError(String),

    #[error("LLM Inference API Error:{0}")]
    APICallError(String)
}

pub struct LLMOrchestrator {
    api_key: String,
    client: RetryableClient,
    system_prompt: String,
    database: Arc<DatabaseService>
}

impl LLMOrchestrator {
    pub async fn new(context: &Context) -> Result<Self, LLMError> {
        // Extract database from context
        let database = context.database.clone();

        // Read API key from environment
        let api_key = env::var("GROQ_API_KEY")
            .map_err(|_| LLMError::MissingApiKey)?;

        // Initialize RetryableClient
        let client = RetryableClient::new();

        // Read system prompt from file
        let system_prompt = fs::read_to_string("assets/llm/system_prompt.txt")
            .map_err(|e| LLMError::SystemPromptReadError(e.to_string()))?;

        Ok(Self {
            api_key,
            client,
            system_prompt,
            database,
        })
    }

    pub async fn try_parse(
        &self,
        request: &str
    ) -> Result<String, LLMError> {

        let model_name = "openai/gpt-oss-20b";
        let response = self
            .client
            .execute_with_retry(
                self.client
                    .post("https://api.groq.com/openai/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .json(&json!({
                        "model": model_name,
                        "messages": [
                            {
                                "role": "system",
                                "content": self.system_prompt.as_str()
                            },
                            {
                                "role": "user",
                                "content": request
                            }
                        ],
                        "tools": tools,
                        "tool_choice": "required",
                        "temperature": 0.0,
                        "max_completion_tokens": 8192
                    })),
            )
            .await
            .map_err(|e| LLMError::APICallError(e.to_string()))?;
        Ok("dummy_response".to_string())
    }

    
}