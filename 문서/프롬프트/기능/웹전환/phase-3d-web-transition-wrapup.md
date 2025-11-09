# Phase 3D – 웹전환 마무리 계획 (Cloud 실행 안정화 + 운영 문서)

## 0. 목표
- Phase 3A~3C에서 구현한 GitHub/Claude 비밀 주입 + docker 실행 경로를 **실제 배포 가능한 상태**로 다듬는다.
- 이미지/CI/문서/테스트/운영 체계를 정리하여 “웹전환 과제 완료” 기준을 명확히 충족한다.

### 진행 상황 (2025-11-09)
- ✅ Claude Runtime Dockerfile/스크립트/CI 워크플로 추가
- ✅ DEPLOYMENT.md + Claude Code 가이드 + 운영 runbook 업데이트
- ✅ CloudContainerService secret 헬퍼 테스트 추가 및 cron/알림 체크리스트 정리
- ⏳ 추가 운영 자동화(알림 시스템 연결)는 차후 작업 항목으로 유지

## 1. 범위
| 영역 | 포함 | 제외 |
| --- | --- | --- |
| 컨테이너 이미지 | `anyon-claude` 베이스 이미지 정의, 빌드 파이프라인, 버전 고정 | CPU/GPU 최적화 등 장기 이슈 |
| 문서/가이드 | DEPLOYMENT/USER 가이드에서 Claude 로그인 + Docker 실행 절차, env/권한, 문제 해결 | 마케팅/세일즈 자료 |
| 테스트 | docker exec 경로(Claude/GitHub 명령) smoke test, secret 디렉터리 정리 확인, UI 흐름 회귀 | 대규모 부하/성능 벤치 |
| 운영/보안 | 시크릿 경로 점검 체크리스트, 로그/알람, 만료 대응 | 정식 SIEM/감사 체계 |

## 2. 세부 계획

### 2.1 컨테이너 이미지 & CI
1. **이미지 명세 정의**
   - 베이스: `debian:bookworm` or 기존 Anyon runtime 이미지 + Node 18 + git + build-essential + Claude CLI + GH CLI.
   - `CloudContainerSettings.default_image` = `registry.example.com/anyon/claude-runtime:<semver>`.
2. **Dockerfile 작성**
   - `RUN npm install -g @anthropic-ai/claude-code@2.0.31 @github/cli`, `git config --global credential.helper store` 제거 등.
   - 비필요 사용자 데이터 제거, 비루트 사용자 (`anyon`) 생성.
3. **CI 파이프라인**
   - GitHub Actions or GCP Cloud Build에서 Dockerfile 빌드 → 컨테이너 레지스트리 푸시.
   - 태그 전략: `main` → `latest`, release → `vX.Y.Z`.
   - 이미지 서명/취약점 스캔(optional) 체크리스트.
4. **배포 구성 업데이트**
   - `CloudContainerSettings` default를 새 이미지 경로로 갱신하고, ENV(`ANYON_CLOUD_IMAGE`)로 override 가능하도록 문서화.

### 2.2 문서 & 운영 가이드
1. **`docs/DEPLOYMENT.md` 업데이트**
   - CloudContainerService 소개, 필요한 env (`ANYON_SECRET_KEY`, `ANYON_TEMP_DIR`, `ANYON_CLOUD_IMAGE`).
   - Docker daemon 요구 사항, 볼륨/권한 설정, secret 디렉터리 경로.
2. **사용자 가이드/FAQ**
   - Claude 로그인 UI 흐름(스크린샷 포함), “로그인 방법 선택 → 브라우저 승인 → Secret 저장” 설명.
   - GitHub 연결: oauth + PAT fallback, token 만료 알림에 대한 안내.
3. **운영 체크리스트**
   - 컨테이너/secret 디렉터리 잔여물 정리 스크립트.
   - 로그/알람: Claude 로그인 실패, docker exec 실패, secret 주입 실패 시 Slack/Email 알림 절차.
   - 만료 대응 프로세스 (Claude token 만료 시 UI 배너 + runbook).

### 2.3 테스트 & 검증
1. **docker exec Smoke Test**
   - CI job에서 `cargo test --features cloud --package services cloud_exec_smoke` (mock DockerHarness) 추가.
   - 실제 VM 시나리오: git fetch + Claude CLI ping 명령이 컨테이너 내부에서 성공하는지 검증.
2. **Secret 정리 테스트**
   - 실행 후 `/tmp/anyon/cloud-secrets/<attempt>`가 삭제되는지 확인하는 통합 테스트.
3. **프런트엔드 회귀**
   - Vitest/Playwright로 Claude 로그인 모달 흐름을 mock SSE로 테스트.
   - General Settings Claude 카드 상태 전환(연결/해제) snapshot 테스트.
4. **수동 QA 체크리스트**
   - Cloud 모드에서 Claude/GitHub 연결 → 작업 실행 → secret 디렉터리 삭제까지 사람이 검증하는 절차 문서화.

### 2.4 보안/운영 보강
1. **시크릿 경로**: `/tmp/anyon/cloud-secrets` 접근 권한 (700) 보장, cron으로 orphan 디렉터리 제거.
2. **로그 마스킹**: Claude/GitHub 토큰 패턴을 `MsgStore` push 전에 필터링하는 middleware 추가 여부 검토.
3. **만료/장애 알림**: SecretStore 조회 실패, docker exec exit code ≠ 0, Claude CLI error 이벤트를 Sentry/Slack으로 알리는 핸들러 등록.

## 3. 일정 (제안)
| 주차 | 작업 | 산출물 |
| --- | --- | --- |
| 1주차 | Dockerfile 작성, 이미지 빌드/배포 CI | Dockerfile, CI job, registry 이미지 |
| 2주차 | 문서/가이드 업데이트 + 운영 체크리스트 | DEPLOYMENT.md, USER_GUIDE, runbook |
| 3주차 | 테스트/QA 자동화, secret cleanup 확인 | smoke tests, QA checklist, CI status |

## 4. 완료 기준 (Definition of Done)
- Cloud 배포 환경에서 `ANYON_CLOUD_IMAGE`가 새 이미지로 설정되어 docker exec 기반 실행이 성공.
- Secret 디렉터리가 실행 후 자동 삭제되고, 문서에 경로/권한/정리 절차가 명시됨.
- DEPLOYMENT/USER 가이드가 최신 플로우(Claude UI, docker exec)를 반영하고 리뷰 완료.
- docker exec smoke test + Claude 로그인 모달 회귀 테스트가 CI에 추가되고 통과.
- 운영 체크리스트(알림, 만료 대응, orphan cleanup)가 문서화되어 온보딩 가능 상태.

## 5. 리스크 & 대응
| 리스크 | 영향 | 대응 |
| --- | --- | --- |
| docker 이미지 빌드 시간/용량 증가 | CI 지연 | multi-stage build, 캐시 활용 |
| Secret 디렉터리 삭제 실패(권한 문제) | 토큰 노출 위험 | 삭제 실패 알림 + cron cleanup 스크립트 |
| Claude/GitHub CLI 버전 충돌 | 로그인/exec 실패 | 이미지에 버전 고정 + 정기 업그레이드 계획 |

## 6. 참고
- Phase 3A/B/C 문서: secret 저장 / Claude 로그인 / docker exec 설계.
- aifactory Claude 로그인 UX 참고 링크.
- DockerHarness POC 문서 (도커-클로드코드/README) – API 사용 예시.

> 이 문서가 “웹전환 과제 마무리”를 위한 실행 계획의 기준선입니다. 각 항목은 JIRA 티켓(or GitHub Issues)로 세분화하여 담당/기한을 지정하고, 완료 시 본 문서의 상태를 업데이트하세요.
