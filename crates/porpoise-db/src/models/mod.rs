pub mod agent;
pub mod config;
pub mod session;
pub mod terminal;
pub mod worktree;

use porpoise_core::error::Result;

pub trait Create<T> {
    fn create(&self, entity: &T) -> Result<T>;
}

pub trait Read<T> {
    fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    fn list(&self) -> Result<Vec<T>>;
}

pub trait Update<T> {
    fn update(&self, id: &str, entity: &T) -> Result<T>;
}

pub trait Delete {
    fn delete(&self, id: &str) -> Result<()>;
}
