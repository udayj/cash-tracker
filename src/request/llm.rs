use crate::configuration::Context;
use crate::core::RetryableClient;
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use std::fs;
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

fn get_tools() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "add_cash",
                "description": "Add or subtract cash from the balance",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "amount": {"type": "number", "description": "Amount to add (positive) or subtract (negative)"},
                        "date": {"type": "string", "description": "Date in dd/mm/yyyy format"}
                    },
                    "required": ["amount", "date"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "add_expense",
                "description": "Add a new expense with description and category",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "amount": {"type": "number", "description": "Expense amount (positive number)"},
                        "description": {"type": "string", "description": "Brief description of the expense"},
                        "category": {"type": "string", "description": "Category name (e.g., Grocery, Food, Transport)"},
                        "date": {"type": "string", "description": "Date in dd/mm/yyyy format"}
                    },
                    "required": ["amount", "description", "category", "date"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "modify_expense",
                "description": "Modify one or more fields of an existing expense atomically. You can update amount, description, category, or date. When amount changes, update description accordingly (e.g., '10 fruits' to '20 fruits').",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "expense_id": {"type": "integer", "description": "ID of the expense to modify"},
                        "amount": {"type": "integer", "description": "New amount in rupees (optional)"},
                        "description": {"type": "string", "description": "New description (optional)"},
                        "category": {"type": "string", "description": "New category (optional)"},
                        "date": {"type": "string", "description": "New date in dd/mm/yyyy format (optional)"}
                    },
                    "required": ["expense_id"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "delete_expense",
                "description": "Delete an expense by ID",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "expense_id": {"type": "integer", "description": "ID of the expense to delete"}
                    },
                    "required": ["expense_id"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_balance",
                "description": "Get the current cash balance",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_expense_breakdown",
                "description": "Get expense breakdown by category for a date range",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "start_date": {"type": "string", "description": "Start date in dd/mm/yyyy format"},
                        "end_date": {"type": "string", "description": "End date in dd/mm/yyyy format"}
                    },
                    "required": ["start_date", "end_date"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_category_expenses",
                "description": "Get all expenses for a specific category in a date range",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "category": {"type": "string", "description": "Category name"},
                        "start_date": {"type": "string", "description": "Start date in dd/mm/yyyy format"},
                        "end_date": {"type": "string", "description": "End date in dd/mm/yyyy format"}
                    },
                    "required": ["category", "start_date", "end_date"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_categories",
                "description": "Get all expense categories with their totals",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        }
    ])
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
