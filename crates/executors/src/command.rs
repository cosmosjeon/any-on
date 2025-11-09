use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use command_group::{AsyncCommandGroup, AsyncGroupChild};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;
use ts_rs::TS;
use workspace_utils::shell::resolve_executable_path;

use crate::executors::ExecutorError;

#[derive(Debug, Error)]
pub enum CommandBuildError {
    #[error("base command cannot be parsed: {0}")]
    InvalidBase(String),
    #[error("base command is empty after parsing")]
    EmptyCommand,
    #[error("failed to quote command: {0}")]
    QuoteError(#[from] shlex::QuoteError),
}

#[derive(Debug, Clone)]
pub struct CommandParts {
    program: String,
    args: Vec<String>,
}

impl CommandParts {
    pub fn new(program: String, args: Vec<String>) -> Self {
        Self { program, args }
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn into_owned(self) -> (String, Vec<String>) {
        (self.program, self.args)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS, JsonSchema, Default)]
pub struct CmdOverrides {
    #[schemars(
        title = "Base Command Override",
        description = "Override the base command with a custom command"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_command_override: Option<String>,
    #[schemars(
        title = "Additional Parameters",
        description = "Additional parameters to append to the base command"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_params: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS, JsonSchema)]
pub struct CommandBuilder {
    /// Base executable command (e.g., "npx -y @anthropic-ai/claude-code@latest")
    pub base: String,
    /// Optional parameters to append to the base command
    pub params: Option<Vec<String>>,
}

impl CommandBuilder {
    pub fn new<S: Into<String>>(base: S) -> Self {
        Self {
            base: base.into(),
            params: None,
        }
    }

    pub fn params<I>(mut self, params: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.params = Some(params.into_iter().map(|p| p.into()).collect());
        self
    }

    pub fn override_base<S: Into<String>>(mut self, base: S) -> Self {
        self.base = base.into();
        self
    }

    pub fn extend_params<I>(mut self, more: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let extra: Vec<String> = more.into_iter().map(|p| p.into()).collect();
        match &mut self.params {
            Some(p) => p.extend(extra),
            None => self.params = Some(extra),
        }
        self
    }

    pub fn build_initial(&self) -> Result<CommandParts, CommandBuildError> {
        self.build(&[])
    }

    pub fn build_follow_up(
        &self,
        additional_args: &[String],
    ) -> Result<CommandParts, CommandBuildError> {
        self.build(additional_args)
    }

    fn build(&self, additional_args: &[String]) -> Result<CommandParts, CommandBuildError> {
        let mut parts = split_command_line(&self.simple_join(additional_args))?;

        let program = parts.remove(0);
        Ok(CommandParts::new(program, parts))
    }

    fn simple_join(&self, additional_args: &[String]) -> String {
        let mut parts = vec![self.base.clone()];
        if let Some(ref params) = self.params {
            parts.extend(params.clone());
        }
        parts.extend(additional_args.iter().cloned());
        parts.join(" ")
    }
}

fn split_command_line(input: &str) -> Result<Vec<String>, CommandBuildError> {
    #[cfg(windows)]
    {
        let parts = winsplit::split(input);
        if parts.is_empty() {
            Err(CommandBuildError::EmptyCommand)
        } else {
            Ok(parts)
        }
    }

    #[cfg(not(windows))]
    {
        shlex::split(input).ok_or_else(|| CommandBuildError::InvalidBase(input.to_string()))
    }
}

pub fn apply_overrides(builder: CommandBuilder, overrides: &CmdOverrides) -> CommandBuilder {
    let builder = if let Some(ref base) = overrides.base_command_override {
        builder.override_base(base.clone())
    } else {
        builder
    };
    if let Some(ref extra) = overrides.additional_params {
        builder.extend_params(extra.clone())
    } else {
        builder
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StdioConfig {
    Inherit,
    Piped,
    Null,
}

impl StdioConfig {
    pub fn piped() -> Self {
        Self::Piped
    }

    pub fn inherit() -> Self {
        Self::Inherit
    }

    pub fn null() -> Self {
        Self::Null
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionCommand {
    program: String,
    args: Vec<String>,
    current_dir: PathBuf,
    env: Vec<(OsString, OsString)>,
    stdin: StdioConfig,
    stdout: StdioConfig,
    stderr: StdioConfig,
    kill_on_drop: bool,
}

impl ExecutionCommand {
    pub fn new(program: String, args: Vec<String>, current_dir: PathBuf) -> Self {
        Self {
            program,
            args,
            current_dir,
            env: Vec::new(),
            stdin: StdioConfig::Inherit,
            stdout: StdioConfig::Inherit,
            stderr: StdioConfig::Inherit,
            kill_on_drop: false,
        }
    }

    pub fn stdin(&mut self, stdio: StdioConfig) {
        self.stdin = stdio;
    }

    pub fn stdout(&mut self, stdio: StdioConfig) {
        self.stdout = stdio;
    }

    pub fn stderr(&mut self, stdio: StdioConfig) {
        self.stderr = stdio;
    }

    pub fn kill_on_drop(&mut self, value: bool) {
        self.kill_on_drop = value;
    }

    pub fn env<K, V>(&mut self, key: K, value: V)
    where
        K: Into<OsString>,
        V: Into<OsString>,
    {
        self.env.push((key.into(), value.into()));
    }

    pub fn args(&mut self, args: &[String]) {
        self.args = args.to_vec();
    }

    pub fn current_dir(&mut self, path: &Path) {
        self.current_dir = path.to_path_buf();
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn args_slice(&self) -> &[String] {
        &self.args
    }

    pub fn current_dir_path(&self) -> &Path {
        &self.current_dir
    }

    pub fn env_vars(&self) -> &[(OsString, OsString)] {
        &self.env
    }

    pub fn stdin_config(&self) -> StdioConfig {
        self.stdin
    }

    pub fn stdout_config(&self) -> StdioConfig {
        self.stdout
    }

    pub fn stderr_config(&self) -> StdioConfig {
        self.stderr
    }

    pub fn should_kill_on_drop(&self) -> bool {
        self.kill_on_drop
    }
}

#[async_trait]
pub trait CommandRuntime: Send + Sync {
    async fn spawn(&self, command: ExecutionCommand) -> Result<AsyncGroupChild, ExecutorError>;
}

pub struct HostCommandRuntime;

#[async_trait]
impl CommandRuntime for HostCommandRuntime {
    async fn spawn(&self, command: ExecutionCommand) -> Result<AsyncGroupChild, ExecutorError> {
        let executable = resolve_executable_path(command.program())
            .await
            .ok_or_else(|| ExecutorError::ExecutableNotFound {
                program: command.program().to_string(),
            })?;

        let mut process = Command::new(executable);
        process.args(command.args_slice());
        process.current_dir(command.current_dir_path());
        apply_stdio(&mut process, command.stdin_config(), IoStream::Stdin);
        apply_stdio(&mut process, command.stdout_config(), IoStream::Stdout);
        apply_stdio(&mut process, command.stderr_config(), IoStream::Stderr);

        if command.should_kill_on_drop() {
            process.kill_on_drop(true);
        }

        for (key, value) in command.env_vars() {
            process.env(key, value);
        }

        let child = process.group_spawn()?;
        Ok(child)
    }
}

#[derive(Clone, Copy)]
enum IoStream {
    Stdin,
    Stdout,
    Stderr,
}

fn apply_stdio(command: &mut Command, config: StdioConfig, stream: IoStream) {
    use std::process::Stdio;

    let stdio = match config {
        StdioConfig::Inherit => Stdio::inherit(),
        StdioConfig::Piped => Stdio::piped(),
        StdioConfig::Null => Stdio::null(),
    };

    match stream {
        IoStream::Stdin => {
            command.stdin(stdio);
        }
        IoStream::Stdout => {
            command.stdout(stdio);
        }
        IoStream::Stderr => {
            command.stderr(stdio);
        }
    }
}
