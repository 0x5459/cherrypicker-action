use std::{ffi::OsStr, future::Future, pin::Pin, process::Output};

use futures::future::BoxFuture;
use tokio::process::Command;

use crate::{DynExecutor, Executor, ExecutorError};

pub struct GitCommandExecutor {
    git: String,
}

impl GitCommandExecutor {
    pub fn new() -> Self {
        Self {
            git: "git".to_string(),
        }
    }
}

impl DynExecutor for GitCommandExecutor {
    fn exec(&self, args: &[&OsStr]) -> BoxFuture<'static, Result<Output, ExecutorError>> {
        let cmd_result = Command::new(&self.git)
            .args(args)
            .spawn()
            .map_err(ExecutorError::SpawnError);
        Box::pin(async move {
            cmd_result?
                .wait_with_output()
                .await
                .map_err(ExecutorError::SpawnError)
                .and_then(|output| {
                    if !output.status.success() {
                        Err(ExecutorError::OutputError(output))
                    } else {
                        Ok(output)
                    }
                })
        })
    }
}
