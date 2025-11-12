# Project Overview: Anyon

## Purpose
Anyon is an orchestration platform for AI coding agents (Claude Code, Gemini CLI, Codex, Amp, etc.). It helps developers:

- Easily switch between different coding agents
- Orchestrate multiple coding agents in parallel or sequence
- Quickly review work and start dev servers
- Track task status across coding agents
- Centralize MCP (Model Context Protocol) configurations
- Open projects remotely via SSH when running on a remote server

## Key Features
- **Multi-Agent Support**: Run different AI agents on different tasks
- **Task Management**: Kanban-style interface for tracking agent work
- **Git Worktree Integration**: Isolated worktrees per task with automatic cleanup
- **MCP Server**: Anyon itself acts as an MCP server with tools like `list_projects`, `list_tasks`, `create_task`
- **Event Streaming**: Real-time process logs via SSE (Server-Sent Events)
- **Remote Development**: SSH integration for remote project access

## Target Users
Software engineers who use AI coding agents extensively and need better orchestration, visibility, and control over agent-driven development workflows.
