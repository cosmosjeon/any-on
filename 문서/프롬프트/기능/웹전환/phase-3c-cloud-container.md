# Phase 3C – CloudContainerService 실행 전환 & 비밀 주입

## 0. 목표
- executor 실행을 호스트 프로세스에서 Docker 컨테이너 내부로 완전히 이동
- GitHub/Claude 비밀을 컨테이너 런타임에 안전하게 주입하고, 실행 종료 시 폐기
- docker exec 기반 프로세스 제어/로그 스트리밍/강제 종료를 제공

## 1. 선행 작업
- Phase 3A SecretStore, Phase 3B Claude 토큰 저장이 완료되어야 함
- `services::docker_poc` 헬퍼(DockerHarness) 안정화 및 오류 처리 재사용

## 2. 설계 변경

### 2.1 ContainerService 계층 개편
- `CloudContainerService`가 `ContainerService` trait를 직접 구현하는 대신, `docker exec` 기반 `CommandRunner`를 주입
- `LocalContainerService::start_execution_inner` 로직을 분리하여 “명령 실행” 부분을 trait로 추상화 (HostRunner vs DockerRunner)
- 실행 시작 시: `docker exec -d <container> bash -lc '<serialized command>'` 형태로 CLI를 실행하고, stdout/stderr는 `attach` API로 스트리밍하여 MsgStore에 기록

### 2.2 GitHub/Claude 비밀 주입
- Runner가 실행되기 전 SecretStore에서 필요한 토큰 목록을 조회
- 컨테이너 내부 `/tmp/secrets` tmpfs에 JSON 파일 또는 env 파일 형태로 쓰고, 실행 시 `env -i` 와 함께 전달
- 실행 종료/오류 시 파일 삭제 확인 (`docker exec rm` + `docker exec shred`)

### 2.3 로그/입력 스트리밍
- `docker attach` 또는 `docker exec --interactive` 를 이용하여 stdout/stderr를 tokio stream으로 읽고, 기존 MsgStore pipeline에 주입
- stdin 필요 시 `docker exec -i` 스트림을 유지하거나, Claude CLI 세션처럼 별도 API에서 stdin 바이트를 전송

### 2.4 강제 종료
- 실행 중인 exec ID를 저장하고, stop 요청 시 `docker exec --signal SIGTERM` → 타임아웃 후 `docker kill` 수행
- 컨테이너 자체가 idle loop를 유지하므로, 작업 종료 후에도 컨테이너는 다음 실행을 위해 재사용; attempt 삭제 시 `docker stop/remove`

### 2.5 이미지/환경 관리
- `CloudContainerSettings`에 `run_image`, `setup_script`(예: `npm install -g @anthropic-ai/claude-code`) 지정
- CI에서 `anyon-claude:latest` 이미지를 빌드/배포하도록 문서화, 버전 태그 사용 고려
- 컨테이너 내 사용자 UID/GID를 `ANYON_DOCKER_USER`와 매칭하여 파일 퍼미션 문제 방지

## 3. 구현 순서
1. `CommandRunner` 추상화 도입 (HostRunner, DockerRunner)
2. Cloud build일 때 `DockerRunner` 사용하도록 LocalDeployment/CloudDeployment 구성 수정
3. docker exec stdout/stderr 스트리밍 + MsgStore 연동
4. Secret 주입 & 정리 로직 추가 (GitHub PAT → git credential helper, Claude 토큰 → `CLAUDE_CONFIG_PATH`)
5. 실행 중 모니터링/강제 종료 API 대응
6. 통합 테스트: docker harness 기반으로 exec 성공/실패/kill 시나리오 검증

## 4. 검증 체크리스트
- `cargo test --features cloud` 에 docker 통합 테스트 추가 (CI에서 optional)
- 실제 VM에서 git clone / Claude 실행 / cleanup e2e
- 비밀 파일이 작업 종료 후 존재하지 않는지 확인 스크립트 작성

## 5. 문서/운영
- `docs/DEPLOYMENT.md` 에 docker exec 기반 실행 흐름, 필요한 env, 이미지 빌드 방법 추가
- 운영 체크리스트: 컨테이너 로그 수집, 디스크 정리, 비밀 경로 점검
