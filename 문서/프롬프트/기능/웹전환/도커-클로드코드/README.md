# Phase 0 – Docker POC 요약

## 목적
- Claude Code 컨테이너를 호스트 Docker Desktop 환경에서 구동하며, Rust 서비스가 Docker API(bollard)를 통해 컨테이너 수명주기를 제어할 수 있는지 검증합니다.
- Phase 1 `CloudContainerService` 구현 전에 필요한 핵심 기능(이미지 준비, 로그 스트리밍, 강제 종료, 리소스 제한, exit code 수집)을 작고 반복 가능한 테스트 하니스로 확립합니다.

## 핵심 산출물
- `crates/services/src/services/docker_poc.rs`
  - `DockerHarness`: 이미지 보장, 컨테이너 생성/시작/중지/삭제, 로그 스트리밍, exec, wait/inspect를 캡슐화.
  - `ContainerExit`: `docker wait` + `docker inspect` 정보를 묶어 exit code, 에러, 종료 시각, OOM 여부를 제공.
  - `stop_container_with_timeout`: 강제 종료 전 grace period를 주입 가능하도록 설계.
  - 통합 테스트 5종(`test_docker_basic`, `test_container_echo_logs`, `test_claude_image_reports_version`, `test_streaming_and_forced_shutdown`, `test_resource_limits_applied`).

- Docker 이미지: `docker/claude-code/Dockerfile` → `docker build -t anyon-claude:latest docker/claude-code`.
- 문서: `phase-0-poc-results.md`(실행 로그/이슈)와 `phase-0-architecture.md`(설계 개요).

## 실행 요약
```bash
docker build -t anyon-claude:latest docker/claude-code
CLANG_MODULE_CACHE_PATH=$(pwd)/target/clang-cache cargo test --package services docker_poc -- --nocapture
```
- 2025-11-08 05:18 KST 기준으로 다섯 테스트 모두 통과(총 10.22 s).
- Docker Desktop 또는 `/var/run/docker.sock`에 접근 가능한 환경이 필요하며, sandbox CLI에서는 권한 부여가 선행되어야 합니다.

## 다음 단계(Phase 1 준비)
1. `DockerHarness`/`ContainerExit` 인터페이스를 `CloudContainerService` 설계로 승계.
2. CI 및 자동화 환경에 Docker 소켓 권한·이미지 프리페치 스크립트를 추가해 재현성을 확보.
3. 로그/exit code 구조를 공유 타입(ts-rs)로 노출해 프런트엔드/CLI에서도 동일 상태를 해석하도록 확장.

