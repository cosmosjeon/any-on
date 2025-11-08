# Phase 3A – GitHub 인증 & Secret Store 마이그레이션

## 0. 목표 (2025-11-09 1차 완료)
- config.json 의존을 제거하고 GitHub OAuth/PAT을 암호화된 SecretProvider(DB)로 이전
- 다중 사용자 환경에서 Device Flow 상태를 안정적으로 관리
- cloud/non-cloud 빌드에 따라 GitHub 기능 노출을 제어하고 만료 UX를 개선
- 마이그레이션 후 `/api/github/*` 엔드포인트와 프런트 GitHub import 플로우를 재검증

### ✅ 현황 요약
- SecretStore(AES-GCM) + `secrets` 테이블 도입 및 마이그레이션 완료
- 앱 기동 시 config.json → SecretStore 이전, GitHub 모든 경로가 SecretStore 토큰 사용
- 프런트 `UserSystemInfo`가 `github_secret_state / is_cloud`를 받아 UI 토글, 만료 UX 개선
- `npm run check`, `cargo check`, `npm run generate-types` 검증 완료

> 📌 후속: secret 키 배포 전략 문서화, PAT 제거 버튼 UX/문구 다국어 번역, SecretStore 재활용(Claude) 연계

## 1. 선행 조건
1. SecretProvider 스키마 및 키 관리 정책 결정 (환경변수 기반 AES-GCM 권장)
2. DB 마이그레이션 파일 및 sqlx 타입 업데이트 준비
3. shared/types.ts 재생성 파이프라인 확인 (`npm run generate-types`)

## 2. 작업 단계

### 2.1 SecretProvider 설계
- `crates/db` 에 `secrets` 테이블 추가: `id`, `user_id`, `provider`, `secret_blob`, `created_at`, `updated_at`
- AES-GCM 암/복호화를 제공하는 `crates/services/src/services/secret_store.rs`(신규) 작성
- 키는 `ANYON_SECRET_KEY`에서 로드, 키 로테이션을 위해 key version 컬럼 포함

### 2.2 Config 구조 개편
- `Config.github` 에서 토큰 필드를 제거하거나 `SecretRef`로 대체, shared/types.ts 업데이트 후 프런트에서 참조 제거
- `config_provider.getConfig` 응답에 “GitHub 연결 여부/username/email”만 남기고 토큰은 SecretStore에서만 조회
- `save_config_to_file` 호출을 SecretStore 업데이트 로직으로 교체 (GitHub 관련 필드 변경 시 SecretStore write)

### 2.3 AuthService & Device Flow 개편
- `AuthService`에 세션별 Device Flow state map 추가 (예: HashMap<SessionId, DeviceFlowState>)
- `/api/auth/github/device/*` 요청시 세션/사용자 식별자를 받아 상태를 저장하고, 동시에 다수 요청 처리 가능하도록 리팩터링
- Github OAuth 완료 시 SecretStore에 암호화 저장 후 Config에 username/email만 반영

### 2.4 config.json → SecretStore 마이그레이션
- 서버 부팅 시 Legacy config를 읽고 GitHub 토큰이 있으면 SecretStore로 옮긴 후 config에서 제거
- 마이그레이션 성공 시 백업 파일(`config.json.bak-<timestamp>`) 생성

### 2.5 cloud Feature Gate & UX
- 프런트 `ProjectFormDialog`, `GeneralSettings`에서 `environment?.isCloud`(또는 새 prop)을 확인해 GitHub import UI 토글
- `githubTokenInvalid` 상태를 GeneralSettings 카드와 ProjectFormDialog 경고에 노출, 재로그인 CTA 연결
- `/api/github/*`는 cloud feature flag가 꺼져 있으면 404 대신 명확한 메시지 반환

### 2.6 검증
- 유닛: SecretStore encrypt/decrypt, AuthService state map
- 통합: Device Flow → Secret 저장 → `/api/github/repositories` 호출
- 프런트 e2e: GitHub 로그인, 토큰 만료 경고, cloud/non-cloud 토글

## 3. 산출물
- SecretProvider 모듈 + DB 마이그레이션
- Config/Shared types 업데이트 및 프런트 수정 PR
- QA 체크리스트 (cloud/local 환경별 GitHub 플로우)

## 4. 위험 & 완화
- **Key 누락**: 서버 기동 시 `ANYON_SECRET_KEY` 없으면 명확한 오류 + 문서화
- **Migration 실패**: 백업 파일 생성 및 롤백 스크립트 제공
- **동시 로그인 Race**: 세션 키 기준으로 상태 저장, 완료/취소 시 map 정리
