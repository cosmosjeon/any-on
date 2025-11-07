# Claude Code Image (Phase 0)

This folder houses the Ubuntu-based Docker image that pre-installs the Claude Code CLI. The image is used in Phase 0 tests to confirm we can run Claude inside an isolated container and capture stdout/stderr from Rust.

## Build

```bash
cd /Users/cosmos/Documents/dev/any-on
docker build -t anyon-claude:latest -f docker/claude-code/Dockerfile .
```

Pass `--build-arg NODE_MAJOR=<version>` if you need to pin Node.js to a different LTS line (default: 20).

## Verify

```bash
docker run --rm anyon-claude:latest claude --version
```

Expected result is version text that begins with `Claude Code`, proving the CLI is installed and runnable without extra configuration.

## Contents
-
- Installs git/curl/vim/ca-certificates + build-essential for parity with current local executor env.
- Installs Node.js from the Nodesource repo and globally installs `@anthropic-ai/claude-code@2.0.31`.
- Sets `/workspace` as the default working directory so subsequent phases can mount project data in a predictable location.

## Troubleshooting
-
- **Network during build:** Ensure the host can reach both `deb.nodesource.com` and `registry.npmjs.org`.
- **Permission errors:** The image expects Docker to run as root (default). If you need a non-root user, extend the image and create the user before running `claude`.
