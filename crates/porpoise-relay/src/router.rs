use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use porpoise_core::{error::Result, state::AppState};

use crate::message::{ErrorCode, ProtocolError, Request, Response, StatusCode};

type HandlerFn = Arc<dyn Fn(Request, AppState) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> + Send + Sync>;

pub struct Router {
    handlers: HashMap<String, HandlerFn>,
    state: AppState,
}

impl Router {
    pub fn new(state: AppState) -> Self {
        Self {
            handlers: HashMap::new(),
            state,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn register(&mut self, method: &str, handler: HandlerFn) {
        self.handlers.insert(method.to_string(), handler);
    }

    pub async fn dispatch(&self, req: Request) -> Response {
        match self.handlers.get(&req.method) {
            Some(handler) => match handler(req.clone(), self.state.clone()).await {
                Ok(body) => Response::ok(req.id, body),
                Err(e) => Response::err(req.id, ErrorCode::Internal, e.to_string()),
            },
            None => Response {
                id: req.id,
                status: StatusCode::NotFound,
                body: serde_json::Value::Null,
                error: Some(ProtocolError {
                    code: ErrorCode::NotFound,
                    message: format!("'{}' not found", req.method),
                    details: None,
                }),
            },
        }
    }
}
