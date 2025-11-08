# Phase 3B – Claude 로그인 & CLI 브릿지 UI

## 0. 목표
- 사용자에게 터미널 없이 Claude Code CLI 로그인을 안내하는 웹 UI 제공
- 백엔드에서 Anthropic OAuth 플로우를 대리 수행하고, SecretStore에 토큰을 저장
- 컨테이너 내부 CLI와 UI 사이에 명령/출력 브릿지를 만들어 선택 입력을 CLI로 전달

### 진행 상황 (2025-11-09)
- SecretStore/SecretState 확장 + `ClaudeAuthManager` 완성: `claude login --stdio` 프로세스 스폰, stdout/stderr SSE 브릿지, stdin 채널/취소, `.claude/meta.json` 추출 및 SecretStore 저장까지 검증 완료
- `/api/auth/claude/*` 엔드포인트 전체 구현 (session/start/stream/input/cancel/logout) — SSE는 JSON 이벤트(`OUTPUT/COMPLETED/ERROR`)로 매핑되고, 입력/취소는 인증 없는 내부 호출에서 동일한 세션 ID를 사용
- 프런트 `ClaudeLoginDialog` (NiceModal)와 General Settings Claude 카드 구축: 스트리밍 로그 뷰어, 선택지 버튼, 자동 브라우저 오픈, 로그 복사, 세션 재시도/중단, SecretState 기반 상태 배지/재연결/로그아웃 UX 제공
- 사용자 설정 캐시(`UserSystemInfo.claude_secret_state`)와 `/api/auth/claude/logout`을 재사용하여 연결/해제 상태가 즉시 반영되며, Phase 3A에서 추가한 암호화 스토어만 토큰을 노출
- 다음 단계는 Phase 3C(작업 컨테이너에 Claude credentials 주입) 및 브라우저 승인 실패 케이스에 대한 통합 테스트 보강

## 1. 플로우 개요
1. 사용자가 General Settings > Integrations > Claude 카드에서 “연결” 버튼 클릭
2. 서버가 CloudContainerService 또는 별도 Claude Auth Runner를 통해 `claude` CLI를 로그인 대기 상태로 실행
3. CLI stdout(텍스트/선택지)을 SSE/WebSocket으로 UI에 스트리밍 → UI는 선택지를 버튼 형태로 표현
4. 사용자가 버튼을 클릭하면 해당 번호/답변이 CLI stdin으로 전달되고, CLI가 브라우저 OAuth URL을 출력
5. UI는 새 탭으로 `https://claude.ai/oauth/...` 혹은 `https://console.anthropic.com/oauth/...` 링크를 자동 오픈
6. 사용자가 브라우저에서 승인하면 CLI가 성공 메시지를 출력하고 access token을 로컬 config에 저장 → 서버가 파일을 읽어 SecretStore에 저장
7. SecretStore에 저장 완료 후 CLI 프로세스 및 임시 파일 제거, UI에 성공 상태 표시

## 2. API/백엔드 작업
- `POST /api/auth/claude/session` : `ClaudeAuthManager`가 `CLAUDE_LOGIN_COMMAND`(기본 `claude login --stdio`)를 temp HOME (`$ANYON_TEMP_DIR/claude-auth/<session>`)에서 실행, stdin/stdout/stderr 파이프 확보 후 세션 ID 반환
- `GET /api/auth/claude/session/:session_id/stream` : BroadcastStream → SSE(EventSource)로 전달, payload는 `OUTPUT { line }`, `COMPLETED { success }`, `ERROR { message }` 중 하나
- `POST /api/auth/claude/session/:session_id/input` : JSON `{ input }`을 받아 tokio mpsc를 통해 CLI stdin으로 전달 (줄바꿈 자동 첨부)
- `POST /api/auth/claude/session/:session_id/cancel` : 세션/프로세스 강제 종료, temp HOME 정리, UI에는 `Session cancelled` 이벤트 push
- `POST /api/auth/claude/logout` : SecretStore(`SECRET_CLAUDE_ACCESS`)의 암호화된 payload 제거로 연결 해제
- SecretStore는 AES-GCM + `ANYON_SECRET_KEY`로 보호되며, CLI가 `.claude/meta.json`에 기록한 credential blob 전체를 저장 → Phase 3C에서 컨테이너 시작 시 동일 blob을 복원할 예정

## 3. 컨테이너/프로세스 관리
- Claude 로그인 전용 경량 컨테이너(또는 host 프로세스) 실행 → `claude login --stdio` 혹은 최신 CLI 명령 사용
- stdout/stderr를 tokio stream으로 읽어 JSON/SSE chunk로 변환 (라인 구분)
- stdin은 mpsc channel로 노출하여 API에서 write
- 세션 타임아웃(예: 5분) 도입, 미완료 시 프로세스 종료 및 SecretStore 기록 제거
- 토큰이 CLI 기본 경로(`~/.claude/meta.json`)에 저장되면, 해당 파일을 읽어 SecretStore에 저장 후 즉시 삭제

## 4. 프런트 UI/UX
- **General Settings Claude 카드 (cloud 전용)**
  - `claude_secret_state.has_credentials`에 따른 상태 배지/설명 텍스트 표시, `Sparkles` 아이콘과 단계별 안내를 보여 줌
  - “Claude 계정 연결/다시 연결” 버튼은 NiceModal(`claude-login`)을 띄우며, 연결된 경우 “연결 해제” 버튼이 SecretStore 삭제 API를 호출
  - 하단에는 사용자 교육용 3단계 리스트(선택지 클릭 → 브라우저 승인 → SecretStore 저장)가 포함되어 “사용자가 코드를 직접 붙여넣지 않는다”는 메시지를 명시
- **ClaudeLoginDialog (NiceModal)**
  - SSE/EventSource를 통해 CLI stdout/stderr를 즉시 표시하는 로그 뷰어(복사 버튼 포함)와 상태 배지를 제공
  - `1) ...` 패턴을 감지해 UI 버튼으로 옵션을 노출, 사용자가 버튼을 클릭하면 `POST /input`으로 숫자를 전달 → 터미널에 직접 입력할 필요 없음
  - CLI 출력에서 `https://claude.ai`/`https://console.anthropic.com` 링크를 감지하면 자동으로 새 탭을 열고, 카드에서도 “브라우저 승인이 필요합니다” 메시지를 강조
  - 수동 입력 인풋은 예외 상황(숫자 외 입력) 대비 fallback 용도로만 제공되며, 토큰/코드 입력을 요구하지 않음
  - “다시 시도”, “세션 중단”, “완료” 버튼과 단계 카드(선택 → 브라우저 승인 → 저장)로 플로우를 시각화하여 사용자에게 현재 단계와 다음 행동을 명확히 안내

## 5. 저장 & 주입
- SecretStore에 `claude_access_token`, `refresh_token`(있다면), `expires_at`, `login_method` 저장
- CloudContainerService는 작업 컨테이너 시작 시 SecretStore에서 토큰을 조회하여 `/tmp/claude-config.json` 작성 후 `CLAUDE_CONFIG_PATH` env로 전달 (Phase 3C 참고)
- 로그/메트릭: 로그인 성공/실패 이벤트 추적, 감사 로그 남김

## 6. 검증 체크리스트
- UI ↔ CLI 스트리밍: 선택-응답 왕복 테스트
- 브라우저 승인 완료 후 SecretStore에 값이 저장되는지 확인
- 컨테이너/프로세스 타임아웃이 제대로 작동하는지 테스트
- CloudContainerService와 연계하여 실제 작업 실행 시 Claude 인증이 필요한 명령에서 401이 발생하지 않는지 확인
- ✅ 2025-11-09: `npm run frontend:check` (tsc --noEmit)로 Claude UI/타입 검사를 통과함

## 7. 문서/운영
- 사용자 가이드에 “클로드 구독 / Anthropic Console 중 선택 → 브라우저에서 승인” 플로우 스크린샷 추가
- 문제 해결 섹션: 브라우저 창이 뜨지 않을 때, 승인 후에도 UI가 멈춘 경우, 재로그인 방법 등
