//! Example local authentication plugin implementation

use super::traits::{AuthPlugin, Plugin};
use async_trait::async_trait;
use std::any::Any;
use std::fmt;

#[derive(Debug)]
pub struct LocalAuthPlugin;

impl Plugin for LocalAuthPlugin {
    fn name(&self) -> &str {
        "local"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl AuthPlugin for LocalAuthPlugin {
    async fn authenticate(&self, username: &str, password: &str) -> Result<bool, String> {
        // TODO: Replace with real user lookup and password verification
        Ok(username == "admin" && password == "password")
    }
}
