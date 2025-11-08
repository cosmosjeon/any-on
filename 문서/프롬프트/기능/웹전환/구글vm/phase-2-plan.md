# Phase 2: GCP VM 배포 상세 계획

## 1. 목표 및 배경
- **목표**: 로컬에서만 동작하던 Anyon(프론트엔드 + 백엔드 + Docker 기반 AI 실행)을 Google Cloud VM(서울 리전)으로 이전하여 어디서나 `http://<VM_IP>:3000`으로 접근 가능하게 만든다.
- **배경**: Phase 0~1에서 Docker 실행 및 CloudDeployment 구조 전환 POC를 마쳤고, 이제 동일 경험을 원격 환경에서도 재현해야 함.
- **성공 기준**:
  1. VM 생성 및 Docker 환경 준비 완료
  2. 빌드 산출물 배포 후 서버 기동, 외부 접속 가능
  3. Task 생성 → Docker 컨테이너 실행 → WebSocket 로그 확인까지 원격에서 정상 동작
  4. 다른 네트워크(모바일 핫스팟 등)에서도 접근 가능

## 2. 범위
- **포함**: GCP VM 생성, 포트/방화벽 설정, 빌드/배포 스크립트 실행, 헬스체크·로그 검증, 문서화/체크리스트 작성
- **제외**: 맞춤 도메인/SSL, GitHub 연동 개선(Phase 3), 멀티 유저 지원, 장기 보안 비밀 관리

## 3. 선행 준비 체크
1. GCP 계정 + 결제 수단 등록, 프로젝트 ID 확보 (`GCP_PROJECT_ID`)
2. 로컬 환경에서 `cargo build --release --features cloud` / `pnpm run dev` 성공 확인
3. `gcloud` CLI 설치, `gcloud auth login` 및 `gcloud config set project <ID>` 완료
4. Docker가 로컬에서 정상 동작하여 Phase 1 결과가 유효함을 확인
5. 네트워크 보안 정책상 3000 포트 개방 가능 여부 확인

## 4. 구현 산출물
| 파일 | 위치 | 내용 |
| --- | --- | --- |
| `create-gcp-vm.sh` | `scripts/deploy/` | n2-standard-4 VM 생성, Docker 설치, 포트 3000 방화벽 규칙 적용 |
| `deploy-to-vm.sh` | `scripts/deploy/` | 백엔드 릴리스/프론트 빌드, 산출물 업로드, Docker 이미지(Claude 툴링) 보장, 서버 기동 |
| `.env.production.template` | `scripts/deploy/` | Anyon 서버가 필요로 하는 핵심 런타임 변수를 템플릿으로 제공 |
| `health-check.sh` | `scripts/deploy/` | HTTP 루트, `/api/projects`, TCP 3000, Docker 서비스 상태 검증 |
| `CHECKLIST.md` | `scripts/deploy/` | 사전 준비 → 배포 → 검증 → 롤백/비용 관리 체크리스트 |
| `DEPLOYMENT.md` | `docs/` | 위 스크립트 실행법, 문제 해결, 유용 명령, 비용 안내 |

## 5. 실행 플로우
1. **VM 생성**
   - `export GCP_PROJECT_ID=<project>`
   - `chmod +x scripts/deploy/create-gcp-vm.sh`
   - `./scripts/deploy/create-gcp-vm.sh`
   - 출력된 외부 IP 기록, 방화벽 규칙 `allow-anyon` 존재 확인
2. **코드 배포**
   - `chmod +x scripts/deploy/deploy-to-vm.sh`
   - `./scripts/deploy/deploy-to-vm.sh <VM_IP>`
   - 로컬에서 `cargo build --release --features cloud`, `pnpm install && pnpm run build`(frontend) 실행
   - VM 내부 `~/anyon` 경로에 서버/정적 자산/마이그레이션 배포 및 `anyon-claude` 이미지 빌드
   - 서버 실행 시 `ANYON_CLOUD_BASE_DIR`, `ANYON_ASSET_DIR`, `ANYON_TEMP_DIR`, `ANYON_WORKTREE_DIR`, `ANYON_DATABASE_FILE`, `ANYON_LOG_FILE`, `ANYON_DOCKER_USER`, `DATABASE_URL` 환경변수를 자동 설정
3. **헬스체크**
   - `chmod +x scripts/deploy/health-check.sh`
   - `./scripts/deploy/health-check.sh <VM_IP>` → HTTP 200, API 200, TCP 3000 open, Docker `active`
4. **기능 검증** (CHECKLIST 참고)
   - `http://<VM_IP>:3000` 접속 → 로그인 → Project/Task 생성 → Task 실행 → Docker 컨테이너 및 WebSocket 로그 확인
   - 모바일 핫스팟 등 외부 네트워크에서 동일 확인
5. **운영 가이드 공유**
   - `docs/DEPLOYMENT.md` 링크와 VM IP, 접근 계정, 로그 확인 커맨드 공유
   - 비용 관리/알림 설정(예: GCP Budget Alert) 안내

## 6. 위험 요소 및 대응
| 위험 | 영향 | 대응 |
| --- | --- | --- |
| CloudDeployment 피처 미구현 | 전용 클라우드 로직 미사용 | 당분간 Local 모드 빌드로 배포, 추후 `cloud` 피처/구현 추가 |
| 방화벽/네트워크 정책 | 외부 접속 불가 | `allow-anyon` 규칙 검증, 추가 IP 제한 시 CIDR 조정 |
| Docker 그룹 권한 | SSH 사용자 docker 명령 실패 | 스타트업 스크립트에서 `usermod -aG docker <기본계정>` 실행 또는 초기에 수동 처리 |
| SQLite 데이터 영속성 | VM 삭제 시 데이터 손실 | Phase 2 완료 후 Persistent Disk 또는 Cloud SQL 로드맵 포함 |
| `nc` 미설치 로컬 환경 | 헬스체크 실패 | README에 netcat 설치 안내 또는 `/dev/tcp` 대체 로직 추가 |
| Claude 설치 시간 | 배포 지연 | `anyon-claude` 베이스 이미지 캐시 전략(Artifact Registry) Phase 2.5에서 검토 |

## 7. 성공 후 기대 결과
- Anyon이 GCP VM에서 24/7 서비스 가능, 어디서나 동일 UI/기능 접근
- Task 실행 시 VM 내부 Docker 컨테이너가 실제로 올라가고 로그 스트림이 브라우저에 표출
- 팀원 누구든 `docs/DEPLOYMENT.md`와 스크립트만으로 동일 배포를 반복 가능
- 비용/롤백 계획이 문서화되어 테스트 종료 시 VM 정리, Phase 3(깃허브 연동) 준비 완료

## 8. 검증 및 보고 절차
1. `scripts/deploy/CHECKLIST.md` 전 항목 체크 후 로그/스크린샷 확보
2. `health-check.sh` 출력과 브라우저 접근 성공 화면을 공유 채널에 보고
3. 문제 발생 시 `docs/DEPLOYMENT.md` Troubleshooting 절차 → 해결 안 되면 로그/명령 내역 첨부하여 이슈화
4. 성공 여부와 VM IP, 차후 계획(Phase 3 일정)을 요약해 문서/회의록에 기록

## 9. 향후 과제(Phase 3 예고)
- GitHub OAuth/Repo clone 기능 Cloud 모드에서 활성화
- 비밀 관리(Claude/GitHub 토큰) 안전 저장소 도입
- CloudDeployment 구현 세부화 및 `DeploymentImpl` 전환
