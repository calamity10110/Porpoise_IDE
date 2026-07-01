use async_trait::async_trait;

use crate::error::Result;
use crate::types::event::SystemEvent;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &SystemEvent) -> Result<()>;

    fn interested_in(&self) -> Vec<String> {
        vec![]
    }
}
