# CloudDeployment 설계 문서 (Draft v0.1)

> 목적: Anyon을 GCP VM 기반 클라우드 환경에서 안정적으로 실행하기 위한 `CloudDeployment` 설계를 정의하고, Local 전용 로직과의 차이를 명확히 한다.

## 1. 배경 및 현재 상태
- 현재 `server` 크레이트는 `DeploymentImpl = local_deployment::LocalDeployment` 로 고정되어 있음.
- 로컬 모드는 사용자 머신의 파일시스템/Git 워크트리/Docker 데몬을 직접 제어한다.
- GCP VM 배포 시에도 동일 로직이 동작하지만, 다음과 같은 제약이 존재:
  - VM 사용자 권한, 디렉터리 레이아웃, 데이터 영속성 등이 로컬 가정과 다를 수 있음.
  - 여러 사용자가 원격으로 접속할 경우 권한 격리, 리소스 관리 이슈가 발생.
  - 로컬 작업과 클라우드 작업을 명확히 구분해 유지보수하기 어려움.

## 2. 목표 및 비범위
| 구분 | 내용 |
| --- | --- |
| **주요 목표** | Anyon 서버가 `--features cloud` 빌드 시 CloudDeployment 구현을 선택하고, GCP VM 환경에 최적화된 파일/컨테이너/비밀 관리 전략을 사용한다. |
| **부가 목표** | LocalDeployment와 CloudDeployment 코드 경계를 명확히 하고, 향후 Kubernetes/다중 VM 확장에 대비한다. |
| **비범위** | FQDN/SSL, 멀티 리전 오토스케일링, 사용자별 격리 등은 후속 페이즈에서 다룬다. |

## 3. 요구사항
### 3.1 기능 요구사항
1. 서버 기동 시 `cloud` feature가 활성화되면 CloudDeployment가 선택되어야 한다.
2. 작업(Task) 실행 시 Docker 컨테이너는 VM에서 실행되며, 로그/파일 diff/메트릭을 기존 API로 노출해야 한다.
3. Git worktree, dev assets, migration DB 파일 등의 경로를 VM 표준 경로(`/var/opt/anyon`, `/home/ubuntu/anyon`)로 설정 가능해야 한다.
4. 웹 UI/CLI가 동일하게 동작하도록 REST/WebSocket/SSE 인터페이스는 변경하지 않는다.

### 3.2 비기능 요구사항
1. 장애 격리: CloudDeployment panic/에러가 LocalDeployment에 영향을 주지 않아야 한다.
2. 성능: Task 실행 대기 시간이 로컬 대비 +10% 이내를 목표로 한다.
3. 보안: Docker 컨테이너 실행 계정, SSH 키, GitHub 토큰 등을 안전한 경로에 저장한다(`/etc/anyon`, Secret Manager 도입 여지).
4. 관측성: Cloud 환경 전용 로그 태그/메트릭(예: `deployment=cloud`)을 추가한다.

## 4. 아키텍처 개요
```
                   ┌────────────────────┐
                   │  Anyon Frontend    │
                   └────────┬───────────┘
                            │HTTP/WebSocket
                   ┌────────▼───────────┐
                   │   Anyon Server     │ (Axum)
                   │ (feature = cloud)  │
                   └────────┬───────────┘
                            │ Deployment trait
             ┌──────────────┴──────────────┐
             │                             │
┌────────────▼───────────┐      ┌──────────▼──────────┐
│ LocalDeployment        │      │ CloudDeployment     │
│ (현재 구현)            │      │ (신규)              │
└────────────┬───────────┘      └──────────┬──────────┘
             │                              │
     ┌───────▼──────┐               ┌──────▼──────┐
     │ Local FS     │               │ Cloud FS    │
     │ Docker (host)│               │ Docker (VM) │
     │ User home    │               │ Service user│
     └──────────────┘               └─────────────┘
```

## 5. CloudDeployment 구성 요소
| 모듈 | 역할 | 구현 계획 |
| --- | --- | --- |
| `cloud_deployment::CloudDeployment` | `Deployment` trait 구현체 | `crates/cloud-deployment` 신규 크레이트 생성, LocalDeployment와 동일 trait 준수 |
| `CloudConfig` | VM 경로, 사용자, 권한 등 설정 | `services::config` 확장 or 별도 구조체 주입. 기존 config 로더(`LocalDeployment::new` 문서 참고)와 동일 훅을 재사용하되, cloud 전용 TOML 섹션/ENV override 추가 |
| `CloudContainerService` | Docker/컨테이너 실행 | 현재 LocalContainerService 복사용으로 시작하되, VM 디렉터리/권한 반영, command 실행 정책(`command.rs`) 맞춤화 |
| `CloudFilesystemService` | 파일 복사/워크트리 관리 | Worktree base 경로, shared assets 경로를 VM 표준으로 변경. `WorktreeManager`/파일 와처(`filesystem_watcher`) 설정도 CloudConfig 기반으로 초기화 |
| `SecretProvider`(TODO) | GitHub/Claude 토큰 저장 | Phase 3에서 Secret Manager 등 연동 예정 |

> 참고: `문서/아키텍쳐/백엔드_아키텍쳐.md` 2.2.5~2.2.7 절에서 LocalDeployment 초기화 순서(`config` 로드 → DB 초기화 → Services 결합 → background tasks)와 각 서비스 모듈 구조를 상세히 정의하고 있으므로 CloudDeployment 도 동일한 순서/인터페이스를 유지해야 한다.

### 5.1 LocalDeployment 대비 변경 포인트 (Architecture 문서 기반)
- **Config 로딩**: 현재 LocalDeployment 는 TOML + env 기반 설정을 읽어 `Services::new` 에 전달한다. Cloud 모드에서는 VM 경로/사용자를 override 할 수 있는 `cloud` 섹션을 TOML 스키마에 추가하고, env (`ANYON_DATA_DIR`, `ANYON_USER`)가 우선하도록 한다.
- **DB 초기화**: Local 모드는 `DBService::new_with_after_connect` 를 호출하며 SQLite 파일을 로컬 경로에 둔다. Cloud 모드는 동일 훅을 재사용하되, CloudConfig 가 지정한 경로(예: `/var/opt/anyon/db/anyon.db`)를 사용하거나 향후 Cloud SQL 커넥션을 선택적으로 열 수 있게 추상화한다.
- **Services wiring**: 백엔드 아키텍처 문서에 나온 `GitService`, `FilesystemService`, `ImageService`, `Approvals` 등은 모두 `LocalDeployment::new` 내부에서 한 번에 생성된다. CloudDeployment도 동일 패턴을 따르되, filesystem/git 서비스에 CloudConfig에서 온 base path, watcher 설정을 주입한다.
- **Background tasks**: LocalDeployment 는 파일 검색 캐시 워밍, orphan worktree 정리 등을 백그라운드로 돌린다. Cloud 모드에서도 동일 작업이 필요하지만, 경로가 변경되었음을 고려해 `WorktreeManager::get_worktree_base_dir()` override 를 제공해야 한다.

## 6. 구현 단계
1. **크레이트 및 피처 스켈레톤**
   - `crates/cloud-deployment` 디렉터리 추가, Cargo workspace 등록.
   - `server`/`services` `Cargo.toml`에 `features = { cloud = ["cloud-deployment"] }` 형태로 정의.
   - `cfg(feature = "cloud")` 조건으로 `DeploymentImpl` 타입 alias 분기.

2. **서비스 연결**
   - Cloud/Local 공통 모듈은 재사용(예: `services::git`), 경로만 CloudConfig로 분기.
   - Docker, Filesystem, Analytics 등 의존성을 Cloud 버전으로 주입.

3. **환경 변수/설정**
   - `.env.production` 또는 systemd unit에서 `ANYON_DEPLOYMENT_MODE=cloud` (feature 플래그와 일치하도록)
   - CloudConfig 예시: `ANYON_DATA_DIR=/var/opt/anyon`, `ANYON_DOCKER_USER=ubuntu`.

4. **CI/빌드**
   - `cargo build --release --features cloud` 잡 추가.
   - 단위 테스트: CloudDeployment 초기화, 기본 서비스 호출 smoke test.

## 7. 데이터/파일 경로 전략
| 리소스 | 로컬 경로 | 클라우드 경로(B 옵션) |
| --- | --- | --- |
| Worktree base | `~/.config/anyon/worktrees` | `/var/opt/anyon/worktrees` (ENV: `ANYON_WORKTREE_DIR`) |
| DB 파일 (SQLite) | `dev_assets/...` | `/var/opt/anyon/data/anyon.db` (ENV: `ANYON_DATABASE_FILE`, 추후 Cloud SQL 고려) |
| Assets | `dev_assets_seed` 복사 | `/var/opt/anyon/data` (ENV: `ANYON_ASSET_DIR`) |
| Temp | OS temp | `/var/opt/anyon/tmp` (ENV: `ANYON_TEMP_DIR`) |
| Logs | worktree/logs | `/var/opt/anyon/logs/server.log` (ENV: `ANYON_LOG_FILE`) |

## 8. 보안 및 권한
- VM OS 사용자: `ubuntu`(UID 1000)를 기본으로 하되, 설정값으로 변경 가능하도록 한다.
- Docker 그룹 추가: 스타트업 스크립트에서만 처리하지 말고 CloudDeployment 초기화 시 검증.
- GitHub/Claude 토큰: Phase 2에서는 `.config/anyon`에 저장(암호화 X), Phase 3에서 Secret Manager 고려.

## 9. 관측성/모니터링
- `tracing` 필터에 `deployment=cloud` 태그를 삽입.
- docker 컨테이너 로그를 `/var/log/anyon/server.log`로 전달하고, 필요 시 Stackdriver Logging 에이전트 설정.

## 10. 롤아웃 플랜
1. **Phase 2.5** – CloudDeployment 스켈레톤 추가, Local과 동일 동작 확인.
2. **Phase 3** – GitHub/Secrets 연동과 함께 Cloud 고유 기능(원격 repo clone, 사용자 계정 분리) 구현.
3. **Phase 4** – Cloud 전용 스케일링/다중 VM 지원 검토.

## 11. 리스크 및 미해결 이슈
| 이슈 | 영향 | 대응/결정 필요 |
| --- | --- | --- |
| SQLite 사용 지속 | VM 삭제 시 데이터 손실 | Cloud SQL or Persistent Disk 이전 일정 정의 |
| Docker 자원 제한 | 다중 Task 실행 시 OOM 가능 | cgroup 제한, 작업 큐 관리 필요 |
| 사용자 격리 | 여러 유저가 같은 VM 사용 시 권한 이슈 | 컨테이너 네임스페이스/별도 VM 전략 협의 |
| Secret 저장소 | 토큰 유출 위험 | GCP Secret Manager 로드맵 수립 |

## 12. 다음 액션 아이템 (냉정 검토)
- [ ] `crates/cloud-deployment` 스켈레톤 생성 및 workspace 등록
- [ ] `server`/`services` Cargo features 정의 (`cloud`)
- [ ] CloudConfig 구조 설계 및 경로/사용자/권한 매핑
- [ ] LocalContainerService 대비 필요한 차이(경로, 권한, 로깅) 목록화
- [ ] SQLite → Cloud SQL 마이그레이션 여부 결정 (Phase 2 범위 외지만 의존성 있음)

> 위 항목을 완료하면 CloudDeployment 구현 착수 기준이 충족되며, 이후 실제 코드 변경을 안전하게 진행할 수 있다.

## 13. 상세 구현 계획 (Step-by-Step)

### 13.1 준비 단계
1. **워크스페이스 정리**
   - `Cargo.toml` 최상단 `members`에 `crates/cloud-deployment` 추가
   - 기존 `crates/local-deployment` 구조를 참고해 신규 크레이트 골격 생성 (`lib.rs`, `container.rs`, `filesystem.rs` 등)
2. **Feature 정의**
   - `crates/server/Cargo.toml`, `crates/services/Cargo.toml` 등에 `features = { cloud = ["cloud-deployment"] }` 선언
   - 기본(default) 피처에서는 포함하지 않고, 빌드 시 `--features cloud`를 명시적으로 요구

### 13.2 CloudDeployment 구현 단계
1. **Config 계층 확장**
   - `CloudConfig` 구조체에서 `ANYON_CLOUD_BASE_DIR`, `ANYON_ASSET_DIR`, `ANYON_TEMP_DIR`, `ANYON_LOG_FILE`, `ANYON_DOCKER_USER`를 읽고 검증한 뒤, 필요한 디렉터리 생성 및 ENV 세팅을 수행한다.
2. **DeploymentImpl 분기**
   - `crates/server/src/lib.rs`에서 `#[cfg(feature = "cloud")] type DeploymentImpl = cloud_deployment::CloudDeployment;` 선언, 기존 Local alias는 `#[cfg(not(feature = "cloud"))]`
   - `main.rs` 등에서 `CloudDeployment::new().await?` 호출 시 Config/Docker 권한 오류를 명확히 로깅
3. **서비스 주입**
   - CloudDeployment 내에서 `DBService::new_with_after_connect`, `GitService::new`, `FilesystemService::new` 등을 초기화하되, CloudConfig 에서 전달받은 경로/사용자 정보를 주입
   - `CloudContainerService`는 Local 버전과 동일 인터페이스를 구현하되, VM 내 표준 디렉터리(`/var/opt/anyon/workspaces/<attempt>` 등) 사용
4. **워크트리/파일 시스템**
   - `WorktreeManager::get_worktree_base_dir`를 CloudConfig 값으로 override (예: `/var/opt/anyon/worktrees`)
   - 파일 와처(`filesystem_watcher`)가 cloud 경로를 감시하도록 설정

### 13.3 검증 단계
1. **단위 테스트**
   - CloudDeployment 초기화 테스트: 기본 Config 로딩 → 경로 생성 여부
   - ContainerService smoke test: dummy 작업 실행이 VM 경로에 디렉터리 생성하는지 확인
2. **통합 테스트**
   - `cargo test --features cloud` 워크플로 추가, CI에서 실행
   - GCP VM 상에서 실제 서버를 `--features cloud`로 빌드/실행 후 Task 생성 → Docker 실행 → 로그 확인
3. **롤백 전략**
   - `feature = cloud`와 무관하게 `local` 모드가 계속 기본이므로, 문제 발생 시 피처 플래그 제거만으로 즉시 기존 동작 복구

### 13.4 문서/운영 업데이트
1. **README/Docs**: Cloud 모드 빌드/실행 방법, 필요한 ENV, 경로 변경 사항 명시
2. **배포 스크립트**: `cargo build --release --features cloud`로 전환, `.env.production.template`에 Cloud 전용 변수 추가
3. **운영 Runbook**: VM 권한/디렉터리 구조, 로그 위치(`/var/log/anyon/server.log`) 등을 운영 문서에 반영

위 Step-by-Step 플랜을 따라 개발하면, 단계별로 검증 가능한 작은 작업 단위로 CloudDeployment를 도입할 수 있다.
