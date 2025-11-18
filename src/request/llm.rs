use crate::configuration::Context;
use crate::core::RetryableClient;
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("Missing GROQ_API_KEY in environment")]
    MissingApiKey,

    #[error("Failed to read system prompt: {0}")]
    SystemPromptReadError(String),

    #[error("LLM Inference API Error:{0}")]
    APICallError(String),

    #[error("Failed to parse LLM response: {0}")]
    ResponseParseError(String),
}

static TOOLS: OnceLock<Value> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
}

#[derive(Debug, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug)]
pub struct LLMResponse {
    pub tool_calls: Vec<ToolCall>,
}

fn get_tools() -> &'static Value {
    TOOLS.get_or_init(|| {
        serde_json::from_str(include_str!("tools.json")).expect("Could not parse tools.json file")
    })
}

pub struct LLMOrchestrator {
    api_key: String,
    client: RetryableClient,
    system_prompt: String,
}

impl LLMOrchestrator {
    pub async fn new(_context: &Context) -> Result<Self, LLMError> {
        // Read API key from environment
        let api_key = env::var("GROQ_API_KEY").map_err(|_| LLMError::MissingApiKey)?;

        // Initialize RetryableClient
        let client = RetryableClient::new();

        // Read system prompt from file
        let system_prompt = fs::read_to_string("assets/llm/system_prompt.txt")
            .map_err(|e| LLMError::SystemPromptReadError(e.to_string()))?;

        Ok(Self {
            api_key,
            client,
            system_prompt,
        })
    }

    pub async fn try_parse(&self, request: &str) -> Result<LLMResponse, LLMError> {
        let model_name = "openai/gpt-oss-20b";
        let tools = get_tools();
        println!("Request:{}", request);
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

        let body: Value = response
            .json()
            .await
            .map_err(|e| LLMError::ResponseParseError(e.to_string()))?;
        println!("{}", serde_json::to_string_pretty(&body).unwrap());
        let message = body["choices"][0]["message"].clone();

        let tool_calls = message["tool_calls"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| serde_json::from_value(tc.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        println!("tool calls:{:#?}", tool_calls);
        Ok(LLMResponse { tool_calls })
    }
}
