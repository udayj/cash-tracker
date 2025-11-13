use crate::{database::DatabaseService, request::llm::LLMOrchestrator};
use std::sync::Arc;
use thiserror::Error;
use crate::configuration::Context;
pub mod llm;

#[derive(Error, Debug)]
pub enum RequestError {

    #[error("Initialization Error")]
    InitializationError
}

pub struct RequestFulfilment {

    pub llm_service: LLMOrchestrator,
    pub database: Arc<DatabaseService>
}

impl RequestFulfilment {

    pub async fn new(context: &Context) -> Result<Self, RequestError> {

        let llm_service = LLMOrchestrator::new(context).await.map_err(|_| RequestError::InitializationError)?;
        let database = context.database.clone();
        Ok(RequestFulfilment {
            llm_service,
            database
        })
    }

    pub async fn fulfil_request(&self, request: &str) -> Result<String, RequestError> {

        let response = self.llm_service.try_parse(request).await.unwrap();
        Ok(response)

    }
}