# Phase 3B – Claude 로그인 & CLI 브릿지 UI

## 0. 목표
- 사용자에게 터미널 없이 Claude Code CLI 로그인을 안내하는 웹 UI 제공
- 백엔드에서 Anthropic OAuth 플로우를 대리 수행하고, SecretStore에 토큰을 저장
- 컨테이너 내부 CLI와 UI 사이에 명령/출력 브릿지를 만들어 선택 입력을 CLI로 전달

### 진행 상황 (2025-11-09)
- SecretStore/SecretState 구조 확장으로 Claude 토큰 보관 준비 완료
- ClaudeAuthManager 초안 작성 중 (CLI spawn/stream, 세션 관리) — API/프런트 연동은 미완
- 프런트/백엔드에 Claude 상태 필드(`claude_secret_state`) 추가해 UI 조건 분기 가능
- 다음 목표: ClaudeAuthManager 마무리 → `/api/auth/claude/*` API → SSE/입력/로그아웃 연결 → NiceModal UI 구현

## 1. 플로우 개요
1. 사용자가 General Settings > Integrations > Claude 카드에서 “연결” 버튼 클릭
2. 서버가 CloudContainerService 또는 별도 Claude Auth Runner를 통해 `claude` CLI를 로그인 대기 상태로 실행
3. CLI stdout(텍스트/선택지)을 SSE/WebSocket으로 UI에 스트리밍 → UI는 선택지를 버튼 형태로 표현
4. 사용자가 버튼을 클릭하면 해당 번호/답변이 CLI stdin으로 전달되고, CLI가 브라우저 OAuth URL을 출력
5. UI는 새 탭으로 `https://claude.ai/oauth/...` 혹은 `https://console.anthropic.com/oauth/...` 링크를 자동 오픈
6. 사용자가 브라우저에서 승인하면 CLI가 성공 메시지를 출력하고 access token을 로컬 config에 저장 → 서버가 파일을 읽어 SecretStore에 저장
7. SecretStore에 저장 완료 후 CLI 프로세스 및 임시 파일 제거, UI에 성공 상태 표시

## 2. API/백엔드 작업
- `/api/auth/claude/session` : POST로 세션 생성 → 내부적으로 컨테이너/프로세스 시작, 세션 ID 반환
- `/api/auth/claude/stream` : SSE/WebSocket으로 CLI stdout 전달
- `/api/auth/claude/input` : 사용자가 선택한 옵션/텍스트를 CLI stdin으로 전달
- `/api/auth/claude/complete` : SecretStore 저장 완료 후 호출(혹은 서버가 자동 전환) → UI에 성공 응답
- 세션 만료/오류 시 `/api/auth/claude/cancel`
- SecretStore 사용: Phase 3A에서 만든 구조 재사용 (provider = `claude`)

## 3. 컨테이너/프로세스 관리
- Claude 로그인 전용 경량 컨테이너(또는 host 프로세스) 실행 → `claude login --stdio` 혹은 최신 CLI 명령 사용
- stdout/stderr를 tokio stream으로 읽어 JSON/SSE chunk로 변환 (라인 구분)
- stdin은 mpsc channel로 노출하여 API에서 write
- 세션 타임아웃(예: 5분) 도입, 미완료 시 프로세스 종료 및 SecretStore 기록 제거
- 토큰이 CLI 기본 경로(`~/.claude/meta.json`)에 저장되면, 해당 파일을 읽어 SecretStore에 저장 후 즉시 삭제

## 4. 프런트 UI/UX
- General Settings → Integrations 섹션에 Claude 카드 추가 (cloud 전용)
- NiceModal 기반 로그인 다이얼로그 구성 요소
  - 단계 표시: “로그인 방법 선택” → “브라우저 승인” → “연결 완료”
  - CLI 출력 영역: monospace 로그 뷰어, 스트리밍된 텍스트 표시
  - 선택지 버튼/입력창: CLI가 `Select login method:` 텍스트를 출력하면 JSON 룰에 따라 버튼 생성
  - 브라우저 승인 URL 수신 시 자동 새 탭 오픈 + 사용자에게 안내 메시지
  - 성공 후 요약(연결 계정, 구독 타입) 표시 및 Close 버튼
- 오류 상태(예: CLI 실패, Anthropic 거절) 시 재시도 UI 제공

## 5. 저장 & 주입
- SecretStore에 `claude_access_token`, `refresh_token`(있다면), `expires_at`, `login_method` 저장
- CloudContainerService는 작업 컨테이너 시작 시 SecretStore에서 토큰을 조회하여 `/tmp/claude-config.json` 작성 후 `CLAUDE_CONFIG_PATH` env로 전달 (Phase 3C 참고)
- 로그/메트릭: 로그인 성공/실패 이벤트 추적, 감사 로그 남김

## 6. 검증 체크리스트
- UI ↔ CLI 스트리밍: 선택-응답 왕복 테스트
- 브라우저 승인 완료 후 SecretStore에 값이 저장되는지 확인
- 컨테이너/프로세스 타임아웃이 제대로 작동하는지 테스트
- CloudContainerService와 연계하여 실제 작업 실행 시 Claude 인증이 필요한 명령에서 401이 발생하지 않는지 확인

## 7. 문서/운영
- 사용자 가이드에 “클로드 구독 / Anthropic Console 중 선택 → 브라우저에서 승인” 플로우 스크린샷 추가
- 문제 해결 섹션: 브라우저 창이 뜨지 않을 때, 승인 후에도 UI가 멈춘 경우, 재로그인 방법 등
