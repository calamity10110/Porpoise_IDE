use async_trait::async_trait;
use serde::Serialize;

use crate::error::Result;

#[async_trait]
pub trait Command: Send + Sync {
    type Output: Serialize;

    fn name(&self) -> &'static str;

    async fn execute(self, state: &crate::state::AppState) -> Result<Self::Output>;
}
