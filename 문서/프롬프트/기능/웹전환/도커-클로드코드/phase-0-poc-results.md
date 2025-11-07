# Phase 0 – Docker POC Results (WIP)

## Environment
- Host: macOS (arm64)
- Docker Desktop: locally accessible via `/var/run/docker.sock`
- Rust toolchain: nightly-2025-05-18 (workspace default)

## Image Build
- Built `anyon-claude:latest` from `docker/claude-code/Dockerfile`.
- Verification: `docker run --rm anyon-claude:latest claude --version` ⇒ `2.0.31 (Claude Code)`.

## Rust POC Code
- Added `bollard` dependency to `crates/services` and scaffolded `services::docker_poc` harness.
- Implemented helper utilities for container lifecycle, log streaming, exec, inspection, and resource limit inspection.
- Hardened the harness so `wait_container` now returns a structured `ContainerExit` (exit code, error text, timestamps, OOM flag) and falls back to `docker inspect` if `docker wait` errors; introduced `stop_container_with_timeout` so tests/services can choose graceful vs. forced shutdown windows explicitly.
- Added Tokio-based tests covering:
  1. Docker daemon connectivity (`test_docker_basic`).
  2. Basic container stdout echo streaming (`test_container_echo_logs`).
  3. Claude image smoke test (`test_claude_image_reports_version`).
  4. Long-running log streaming + forced shutdown (`test_streaming_and_forced_shutdown`).
  5. Resource limit enforcement check (`test_resource_limits_applied`).

## Test Execution Status
- `cargo test --package services docker_poc::tests::test_docker_basic -- --nocapture` → ✅ (ran before the extended suite was added).
- Full suite command: `CLANG_MODULE_CACHE_PATH=... cargo test --package services docker_poc -- --nocapture`.
  - Initial attempts failed with `No space left on device (os error 28)` while linking proc-macro build scripts. Relocating `CARGO_TARGET_DIR`/`TMPDIR` did not help, so disk cleanup was performed.
  - After cleanup the build completed, but every test failed with `Failed to fetch Docker version (Timeout error)` because Docker CLI commands (e.g. `docker ps`) now hang for >20 s.
  - 현재는 Docker Desktop 자체가 실행되지 않아(앱 실행 시도 시 아무 반응 없음) 데몬이 아예 떠 있지 않은 상태. 재부팅 또는 Docker 재설치가 필요할 수 있음.
  - 2025-11-08 04:32 KST 기준 추가 진단:
    - `docker version`은 클라이언트 정보까지는 출력하지만 `unix:///Users/cosmos/.docker/run/docker.sock` 접속 시 `operation not permitted`로 실패.
    - `docker context ls` 결과 현재 컨텍스트는 `desktop-linux`. `docker context use default` 시도 시 `~/.docker/config.json`을 열 수 없어 동일하게 권한 오류 발생.
    - CLI 환경이 Docker 소켓 및 설정 파일에 쓰기 권한을 갖지 못해 데몬 접근 자체가 막혀 있음. 권한/컨텍스트 문제를 해결하지 않으면 테스트 재실행이 불가능.
  - 같은 날 04:45 KST경 호스트 셸에서 Docker Desktop을 재기동하고 `docker context use default`를 완료한 뒤 다음 커맨드를 실행:
    ```
    CLANG_MODULE_CACHE_PATH=$(pwd)/target/clang-cache cargo test --package services docker_poc -- --nocapture
    ```
    - `test_docker_basic`, `test_container_echo_logs`, `test_claude_image_reports_version`, `test_resource_limits_applied` → ✅ (총 10.28 s 내 통과).
    - `test_streaming_and_forced_shutdown`만 `Error: Docker container wait error`로 실패.
    - 동일 테스트를 sandboxed CLI에서 단독 실행하면 `/var/run/docker.sock` 접근 시 `Failed to fetch Docker version (Operation not permitted)`가 즉시 재현되어, 현재 자동화 환경에서는 Docker 데몬 접근권이 여전히 없음.
  - 04:50 KST 추가 재현 (`RUST_LOG=debug CLANG_MODULE_CACHE_PATH=$(pwd)/target/clang-cache cargo test --package services docker_poc::tests::test_streaming_and_forced_shutdown -- --nocapture`):
    - 실행 시간 약 10 s, 말미에 동일한 `Docker container wait error`.
    - `docker logs docker-poc-cancel-…` 출력은 `tick` 4회까지 존재해 stop 호출(3회차)보다 한 사이클 늦게 중단됨을 확인.
    - `docker inspect docker-poc-cancel-…` 결과 ExitCode 137(SIGHUP/STOP 후 SIGKILL). 현재 `wait_container` 로직이 이 종료 코드를 오류로 간주하면서 테스트가 실패.
    - `/var/run/docker.sock`은 `/Users/cosmos/.docker/run/docker.sock`으로 symlink되어 있으며 소유자 `root:daemon`, 권한 `lrwxr-xr-x@`라서 sandbox된 CLI 유저는 여전히 접근 불가.
  - 04:55 KST 코드 패치:
    - `DockerHarness::wait_container`가 이제 컨테이너의 실제 exit code를 반환하고, 호출부는 기대값을 명시적으로 assert.
    - `test_streaming_and_forced_shutdown`은 cancel 케이스에서 exit code 137 또는 143을 허용하도록 갱신.
    - 새 로직이 반영된 `cargo check -p services`는 통과; Docker 데몬 접근이 가능한 호스트에서 전체 테스트를 재실행해 성공 여부를 확인해야 함.
  - 05:05 KST 전체 재실행 (`cargo test --package services docker_poc -- --nocapture`):
    - `test_docker_basic`, `test_container_echo_logs`, `test_resource_limits_applied` → ✅.
    - `test_claude_image_reports_version` → `Docker image anyon-claude:latest not found locally` (이미지가 정리된 듯 하여 `docker/claude-code/Dockerfile`에서 재빌드 필요).
    - `test_streaming_and_forced_shutdown` → `Failed to wait for container … completion` 재발. 이번에는 `wait_container`가 exit code를 받기 전에 Bollard가 `Docker container wait error`를 반환함. 실제 컨테이너/데몬 로그 수집 필요.
  - 05:10 KST 코드 2차 패치:
    - `wait_container`가 `docker wait` 단계에서 오류를 받으면 `docker inspect`를 통해 exit code를 재확인한 뒤 반환하도록 폴백 경로를 추가.
    - `inspect_container`에서 exit code를 읽지 못하는 경우에만 실패로 간주하므로, `Docker container wait error`가 발생해도 컨테이너 상태를 활용할 수 있음.
    - 여전히 실제 Docker 로그와 exit code가 기대와 일치하는지 확인해야 하므로 추가 테스트 필요.
  - 05:18 KST 전체 재실행(`docker build -t anyon-claude:latest docker/claude-code`로 이미지 복구 후 `cargo test --package services docker_poc -- --nocapture`):
    - 다섯 개 테스트 모두 ✅, 총 10.22 s 소요.
    - wait 폴백 + exit code 검증이 의도대로 동작함을 확인. 향후 회귀 방지를 위해 이 흐름을 Phase 1 문서/테스트 가이드에도 공유 필요.
  - Action items:
    1. Docker Desktop/daemon이 정상 기동되어 `docker ps`가 즉시 응답하도록 복구.
    2. Docker 접근이 회복되고 디스크 여유가 유지되면 `cargo test --package services docker_poc -- --nocapture` 재실행 및 로그/타이밍 수집. (부분 성공 기록 있음 — forced shutdown 케이스 원인 파악 필요.)
    3. CLI에서 `/var/run/docker.sock`(또는 `~/.docker/run/docker.sock`)에 접근할 수 있도록 컨텍스트와 파일 권한을 조정해 자동화 환경에서도 재현 가능하게 만들 것.

## Streaming Observations (pending rerun)
- Countdown test captures ~1-second cadence across five log entries and records timestamps for later analysis.
- Cancellation test stops the container after three `tick` messages and confirms clean teardown.
- Need a successful test run to capture actual timing deltas for documentation.

## Issues & Mitigations
| Issue | Status | Mitigation |
| --- | --- | --- |
| Docker container names colliding between retries | ✅ fixed | Added per-test UUID-based names via `unique_name()` helper. |
| mac-notification-sys build failures (Cocoa module cache perms) | ✅ fixed | Set `CLANG_MODULE_CACHE_PATH` inside workspace before invoking Cargo. |
| Disk space exhaustion during Cargo builds | ⚠️ monitored | Cleanup enabled the 2025-11-08 suite run; keep `target/`/`/var/folders/...` under watch so future builds don't regress. |
| Docker socket permission denied from sandboxed CLI | ⏳ outstanding | Even after switching to the `default` context, `/var/run/docker.sock` yields `Failed to fetch Docker version (Operation not permitted)` when invoked from the sandbox. Need a privileged context or socket bind-mount for automated runs. |
| `test_streaming_and_forced_shutdown` ends with `Docker container wait error` | ✅ fixed | `wait_container` fallbacks + image rebuild verified the entire docker_poc suite (10.22 s) on 2025-11-08 05:18 KST. |
| `anyon-claude:latest` image missing locally | ✅ fixed | Rebuilt via `docker build -t anyon-claude:latest docker/claude-code` prior to the successful suite run. |

## Next Steps
1. Free disk space and rerun `cargo test --package services docker_poc -- --nocapture` to capture actual timing data + confirm all tests pass together.
2. Extend docs with measured timings and logs once the suite completes.
3. Promote working helpers into a future `CloudContainerService` implementation (Phase 1).
4. Document the 05:18 KST passing run (timings, exit codes) in Phase 1 planning notes and ensure automation can access the Docker socket going forward.

## Architecture Notes (Phase 0 wrap-up)
- `DockerHarness::wait_container` now yields a `ContainerExit` struct populated via `docker wait` + `docker inspect`, so callers can assert against exit code, OOM state, and completion timestamps instead of raw `i64`s.
- `stop_container_with_timeout` exposes the Docker stop grace period as an argument (default remains 1 s via `stop_container`), making it easier to model graceful shutdown vs. forced termination in future services.
- Tests assert on `ContainerExit` helpers (`succeeded`, `was_signaled`) which doubles as executable documentation for the desired lifecycle semantics heading into Phase 1.
