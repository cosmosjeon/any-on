# Phase 0 – Docker POC Architecture

## 목적 및 범위
- Claude Code 컨테이너를 호스트 Docker Desktop 환경에서 구동·관찰할 수 있는 최소 실행 경로를 마련한다.
- Rust 서비스(`crates/services`)에서 Docker API를 호출하는 공용 하니스(`DockerHarness`)를 설계해 이후 Phase 1의 `CloudContainerService` 구현에 활용 가능한 패턴을 확립한다.
- 로그 스트리밍, 강제 종료, 리소스 제한 등 실제 서비스 시나리오에서 요구되는 핵심 행위를 통합 테스트(Test-driven spike)로 검증한다.

## 구성 요소
| 구성 요소 | 위치 | 역할 |
| --- | --- | --- |
| Docker Harness | `crates/services/src/services/docker_poc.rs` | bollard 클라이언트를 감싸 이미지 보장, 컨테이너 생성/시작/중지/삭제, 로그 스트리밍, exec, wait/inspect 등을 제공 |
| ContainerExit | 동일 파일 | `docker wait` + `docker inspect`에서 수집한 종료 코드, 에러 메시지, 종료 시각, OOM 여부를 단일 구조체로 노출 |
| Tests (`docker_poc::*`) | 동일 파일 | Harness 동작을 통합 테스트(5개)로 검증. 네이밍: `test_docker_basic`, `test_container_echo_logs`, `test_claude_image_reports_version`, `test_streaming_and_forced_shutdown`, `test_resource_limits_applied` |
| Claude Code Image | `docker/claude-code/Dockerfile` → `anyon-claude:latest` | Phase 0 전용 이미지. `claude --version`을 통해 바이너리 정상 동작을 확인 |
| 문서 | `phase-0-poc-results.md`, 본 문서 | 환경 제약, 실행 로그, 아키텍처 메모 기록 |

## 동작 흐름
1. **연결 초기화**  
   - `DockerHarness::connect()`가 `Docker::connect_with_local_defaults()`를 통해 `/var/run/docker.sock` 혹은 Docker Desktop context에 연결하고 즉시 `version()` 호출로 헬스체크.

2. **이미지 확보**  
   - `ensure_image()`가 `docker pull`에 해당하는 `create_image` 스트림을 재시도(기본 3회)하며, 실패 시 경고 로그와 함께 대기 후 재시도.
   - Claude 테스트는 수동으로 `docker build -t anyon-claude:latest docker/claude-code`를 실행해야 하며, 이미지 부재 시 테스트에서 명시적으로 실패한다.

3. **컨테이너 수명주기**  
   - `create_container()`는 공통 설정(Stdout/Stderr attach, TTY off)을 적용해 컨테이너를 생성.
   - `start_container()`/`stop_container_with_timeout()`은 bollard API를 thin wrapper로 감싸며, stop timeout은 `Duration`으로 주입 가능(`stop_container()`는 1초 기본값을 사용).
   - `ContainerGuard`가 Drop 시점에도 force remove를 보장해 테스트 실패 시 리소스 누수가 없도록 함.

4. **로그 스트리밍**  
   - `stream_logs()`는 `docker logs --follow`에 해당하는 스트림을 반환하며, STDOUT/STDERR/Console을 `LogChunk` enum으로 구분한다.
   - 테스트에서는 log 스트림을 소비하면서 실시간 타임스탬프(카운트다운) 또는 특정 문자열(claude version, tick)을 검증한다.

5. **종료 감지 (`wait_container`)**  
   - 우선순위: `docker wait` → `ContainerWaitResponse`에서 exit code/에러 메시지 확보.
   - `docker wait`가 `Docker container wait error`를 내면 `docker inspect`로 fallback: `ContainerExitState`를 통해 exit code, finished_at, OOM flag를 재구성.
   - 반환 타입 `ContainerExit`은 `succeeded()`/`was_signaled()` helper를 제공하여 테스트 및 향후 서비스 레이어가 의사결정을 쉽게 할 수 있도록 함.

6. **테스트 시나리오**  
   - **연결 기본기**: `test_docker_basic`이 `Docker.version()` API 응답을 검증.  
   - **Echo 로그**: 단순 bash echo로 stdout 수집 경로 검증.  
   - **Claude 버전**: 빌드된 이미지에서 `claude --version` 출력이 기대 문자열(`Claude Code`)을 포함하는지 검사.  
   - **Streaming + 강제 종료**: 두 개 컨테이너(카운트다운/long-running)를 이용해 로그 cadence와 `stop_container_with_timeout` -> `ContainerExit::was_signaled()` 흐름을 검증.  
   - **Resource limits**: `HostConfig`에 메모리/CPU 제한을 넣고 `inspect_container`로 적용 여부를 확인.

## 오류 처리 및 진단 포인트
- **Docker Desktop 비가동**: `DockerHarness::connect` 초기화 단계에서 즉시 실패하며 `Failed to fetch Docker version` 메시지로 표면화.
- **소켓 권한 부족**: Phase 0 환경에서는 `/var/run/docker.sock -> ~/.docker/run/docker.sock` 심볼릭 구조로 인해 sandbox CLI가 접근하지 못한다. 테스트 실행 전 사용자 셸에서 권한을 조정하거나 context를 전환해야 한다.
- **이미지 부재**: `assert_local_image`가 404를 즉시 보고하며, 문서에서 재빌드 명령을 안내.
- **wait 타임아웃**: `ContainerExit`에 wait 에러와 inspect 결과가 모두 들어가기 때문에 로그만으로도 종료 경로를 역추적할 수 있다. Phase 1에서는 이 구조체를 상위 서비스 로깅에도 재사용할 계획.

## 종속성 및 빌드 명령
- Rust toolchain: `nightly-2025-05-18`
- 의존 crate: `bollard`, `tokio`, `futures`, `uuid`, `anyhow`, `tracing`
- Docker 이미지:
  - `ubuntu:22.04`
  - `anyon-claude:latest` (`docker/claude-code`에서 수동 빌드)
- 명령:
  ```bash
  docker build -t anyon-claude:latest docker/claude-code
  CLANG_MODULE_CACHE_PATH=$(pwd)/target/clang-cache cargo test --package services docker_poc -- --nocapture
  ```

## Phase 1 대비 사항
- `stop_container_with_timeout`을 활용해 서비스별 Grace period 정책을 외부 설정으로 노출할 수 있도록 Config/Env 연결 필요.
- `ContainerExit` 구조체를 gRPC/HTTP 응답 모델(예: 향후 CloudContainerService)에도 공유하면 양쪽 언어에서 동일한 종료 정보를 해석할 수 있다.
- CI/Automation에서는 Docker 소켓 권한을 명시적으로 세팅하는 스크립트(`scripts/setup-docker-env.sh` 예정)를 추가해 Phase 0에서 겪은 수동 조치를 제거해야 Phase 1 개발 흐름이 매끄럽다.

