use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use workspace_utils::shell::get_shell_command;

use crate::{
    actions::Executable,
    approvals::ExecutorApprovalService,
    command::{CommandRuntime, ExecutionCommand, StdioConfig},
    executors::{ExecutorError, SpawnedChild},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub enum ScriptRequestLanguage {
    Bash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub enum ScriptContext {
    SetupScript,
    CleanupScript,
    DevServer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct ScriptRequest {
    pub script: String,
    pub language: ScriptRequestLanguage,
    pub context: ScriptContext,
}

#[async_trait]
impl Executable for ScriptRequest {
    async fn spawn(
        &self,
        current_dir: &Path,
        _approvals: Arc<dyn ExecutorApprovalService>,
        runtime: &dyn CommandRuntime,
    ) -> Result<SpawnedChild, ExecutorError> {
        let (shell_cmd, shell_arg) = get_shell_command();
        let mut exec_command = ExecutionCommand::new(
            shell_cmd,
            vec![shell_arg.to_string(), self.script.clone()],
            current_dir.to_path_buf(),
        );
        exec_command.kill_on_drop(true);
        exec_command.stdin(StdioConfig::Null);
        exec_command.stdout(StdioConfig::piped());
        exec_command.stderr(StdioConfig::piped());

        let child = runtime.spawn(exec_command).await?;

        Ok(child.into())
    }
}
