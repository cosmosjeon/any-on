# Any-on → Replit 스타일 아키텍처 전환 계획서

**버전:** 1.0
**작성일:** 2025-01-12
**대상 환경:** GCP VM (anyon, 34.50.24.115)
**예상 기간:** 2-3주

---

## 📑 목차

1. [프로젝트 개요](#프로젝트-개요)
2. [현재 상태 분석](#현재-상태-분석)
3. [목표 아키텍처](#목표-아키텍처)
4. [Phase별 구현 계획](#phase별-구현-계획)
   - [Phase 1: 컨테이너 수명 변경](#phase-1-컨테이너-수명-변경)
   - [Phase 2: GCS 스토리지 연동](#phase-2-gcs-스토리지-연동)
   - [Phase 3: Worktree 컨테이너 내부 관리](#phase-3-worktree-컨테이너-내부-관리)
   - [Phase 4: 모니터링 & 최적화](#phase-4-모니터링--최적화)
5. [롤백 계획](#롤백-계획)
6. [체크리스트](#체크리스트)
7. [예상 효과](#예상-효과)

---

## 프로젝트 개요

### 목표
Task 단위 임시 컨테이너 → Project 단위 장기 실행 컨테이너로 전환하여 Replit 스타일의 개발 환경 구축

### 주요 개선사항
- ✅ 프로젝트별 컨테이너 격리 (멀티테넌시)
- ✅ 환경 유지로 빠른 작업 전환 (npm install 등 1회만)
- ✅ GCS 기반 영구 스토리지로 VM 독립성 확보
- ✅ 데이터 손실 위험 제거

### 난이도
⭐⭐☆☆☆ (5점 만점에 2점)

### 환경 정보
- **VM IP:** 34.50.24.115
- **VM 이름:** anyon
- **클라우드:** Google Cloud Platform
- **기존 디렉터리:** `/var/opt/anyon/`

---

## 현재 상태 분석

### 현재 아키텍처

```
사용자 → VM (34.50.24.115)
         └─ /var/opt/anyon/
             ├─ workspace/          ← 프로젝트 원본 (영구)
             │   └─ project-123/
             ├─ worktrees/          ← Task 작업공간 (임시)
             │   ├─ task-1/
             │   └─ task-2/
             └─ data/
                 └─ anyon.db

컨테이너:
- Task Attempt당 1개 생성
- 작업 완료 시 삭제
- 매번 환경 초기화
```

### 문제점

| 문제 | 영향 | 심각도 |
|------|------|--------|
| Task마다 컨테이너 생성/삭제 | 10초+ 지연, 리소스 낭비 | 높음 |
| 환경 초기화 반복 | npm install 등 매번 실행 | 높음 |
| 멀티테넌시 미지원 | 사용자 격리 불가능 | 치명적 |
| VM 종속성 | VM 장애 시 데이터 손실 | 높음 |
| 확장성 부족 | 스케일링 어려움 | 중간 |

---

## 목표 아키텍처

### Replit 스타일 구조

```
사용자 → GCS 버킷: gs://anyon-projects
         └─ user-A/
             ├─ project-123/  ← 프로젝트 영구 저장
             │   ├─ .git/
             │   ├─ src/
             │   └─ .git/worktrees/
             │       ├─ task-1/
             │       └─ task-2/
             └─ project-456/

         ↓ gcsfuse 마운트

         VM (34.50.24.115)
         └─ /var/opt/anyon/projects/  ← GCS 마운트
             └─ user-A/
                 └─ project-123/

         ↓ Docker 마운트

         컨테이너 (장기 실행)
         ├─ project-123 (User A)  ← sleep infinity
         ├─ project-456 (User A)
         └─ project-789 (User B)
```

### 실행 흐름

```
1. 프로젝트 생성
   → GCS에 디렉터리 생성
   → 컨테이너 생성 (sleep infinity)

2. Task 실행
   → 기존 컨테이너 재사용
   → docker exec project-123 claude-code "작업"

3. Idle 관리
   → 30분 비활동 → 컨테이너 중지
   → 재활동 시 → 자동 재시작

4. 프로젝트 삭제
   → 컨테이너 삭제
   → GCS 디렉터리 정리
```

---

## Phase별 구현 계획

### Phase 1: 컨테이너 수명 변경 (3일)

**목표:** Task 단위 → Project 단위 컨테이너

#### 1.1 코드 수정 계획

**A. `crates/services/src/services/cloud_container.rs`**

```rust
// 현재 구조
#[derive(Clone)]
pub struct CloudContainerService<T> {
    provisioned: Arc<DashMap<Uuid, ProvisionedContainer>>,  // task_attempt.id
}

// 변경 후
#[derive(Clone)]
pub struct CloudContainerService<T> {
    provisioned: Arc<DashMap<Uuid, ProvisionedContainer>>,  // project.id
    last_activity: Arc<DashMap<Uuid, SystemTime>>,  // 추가: 활동 추적
}
```

**B. 주요 메서드 변경**

```rust
// 기존: Task Attempt당
async fn ensure_runner(
    &self,
    task_attempt: &TaskAttempt,
    worktree_path: &Path,
) -> Result<String, ContainerError>

// 변경: Project당
async fn ensure_project_container(
    &self,
    project: &Project,
) -> Result<String, ContainerError> {
    // 1. project.id로 컨테이너 찾기
    if let Some(entry) = self.provisioned.get(&project.id) {
        let container_id = entry.container_id.clone();

        // 컨테이너 살아있는지 확인
        if self.docker.inspect_container(&container_id).await.is_ok() {
            self.last_activity.insert(project.id, SystemTime::now());
            return Ok(container_id);
        }

        self.provisioned.remove(&project.id);
    }

    // 2. 새 컨테이너 생성 (장기 실행)
    let project_path = PathBuf::from(&project.git_repo_path);
    let container_id = self.docker.create_container(
        &format!("project-{}", project.id),
        &self.settings.default_image,
        Some(vec!["sleep".into(), "infinity".into()]),  // ← 영구 실행
        Some(HostConfig {
            binds: Some(vec![
                format!("{}:/workspace:rw", project_path.display()),
            ]),
            ..Default::default()
        }),
        ..
    ).await?;

    self.docker.start_container(&container_id).await?;
    self.provisioned.insert(project.id, ..);
    self.last_activity.insert(project.id, SystemTime::now());

    Ok(container_id)
}
```

**C. Idle 컨테이너 관리**

```rust
impl<T> CloudContainerService<T> {
    /// 30분 이상 비활동 컨테이너 중지
    pub async fn cleanup_idle_containers(&self, idle_timeout: Duration) {
        let now = SystemTime::now();

        for entry in self.last_activity.iter() {
            let project_id = entry.key();
            let last_active = entry.value();

            if let Ok(elapsed) = now.duration_since(*last_active) {
                if elapsed > idle_timeout {
                    tracing::info!(
                        project_id = %project_id,
                        idle_minutes = elapsed.as_secs() / 60,
                        "Stopping idle container"
                    );

                    if let Some(container) = self.provisioned.get(project_id) {
                        self.docker
                            .stop_container_with_timeout(
                                &container.container_id,
                                Duration::from_secs(10)
                            )
                            .await
                            .ok();
                    }
                }
            }
        }
    }

    /// 주기적으로 cleanup 실행 (5분마다)
    pub fn spawn_idle_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));

            loop {
                interval.tick().await;
                self.cleanup_idle_containers(Duration::from_secs(1800)).await;
            }
        });
    }
}
```

**D. 삭제 시점 변경**

```rust
// Task 끝나도 컨테이너 유지
async fn delete_inner(&self, task_attempt: &TaskAttempt) -> Result<(), ContainerError> {
    // Worktree만 정리
    self.inner.delete_inner(task_attempt).await
}

// 프로젝트 삭제 시에만 컨테이너 삭제
pub async fn delete_project(&self, project_id: &Uuid) -> Result<(), ContainerError> {
    if let Some((_, container)) = self.provisioned.remove(project_id) {
        self.docker
            .stop_container_with_timeout(&container.container_id, Duration::from_secs(10))
            .await
            .ok();

        self.docker
            .remove_container(&container.container_id, true)
            .await
            .ok();

        tokio::fs::remove_dir_all(&container.secret_dir).await.ok();
    }

    self.last_activity.remove(project_id);
    Ok(())
}
```

#### 1.2 초기화 코드 수정

**`crates/cloud-deployment/src/lib.rs`:**

```rust
impl CloudDeployment {
    pub async fn new(cloud_config: CloudConfig) -> Result<Self, DeploymentError> {
        // ... 기존 코드 ...

        let container_service = CloudContainerService::new(
            local_container_service,
            secret_store.clone(),
            user_id.clone(),
            settings,
        ).await?;

        // Idle cleanup 백그라운드 작업 시작
        let container_service_clone = Arc::new(container_service.clone());
        container_service_clone.spawn_idle_cleanup_task();

        Ok(Self {
            inner: local,
            container_service,
            cloud_config,
        })
    }
}
```

#### 1.3 테스트 시나리오

```bash
# 1. 프로젝트 생성
curl -X POST http://34.50.24.115/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name": "test-project", "git_repo_path": ""}'

# 2. Task 1 실행
curl -X POST http://34.50.24.115/api/tasks \
  -d '{"project_id": "xxx", "description": "task 1"}'

# 3. 컨테이너 확인 (같은 컨테이너 사용 확인)
ssh anyon@34.50.24.115 "docker ps | grep project-"

# 4. Task 2 실행
curl -X POST http://34.50.24.115/api/tasks \
  -d '{"project_id": "xxx", "description": "task 2"}'

# 5. 컨테이너 개수 확인 (1개여야 함)
ssh anyon@34.50.24.115 "docker ps | grep project- | wc -l"

# 6. Idle 타임아웃 테스트 (30분 후)
ssh anyon@34.50.24.115 "docker ps -a | grep project-"
# STATUS가 "Exited"여야 함
```

---

### Phase 2: GCS 스토리지 연동 (4일)

**목표:** VM 로컬 디스크 → GCS 네트워크 스토리지

#### 2.1 GCS 버킷 생성

```bash
# 버킷 생성
gsutil mb -p your-project-id -l asia-northeast3 gs://anyon-projects

# 버전 관리 활성화 (백업용)
gsutil versioning set on gs://anyon-projects

# 수명 주기 정책 (30일 후 삭제된 파일 정리)
cat > lifecycle.json << 'EOF'
{
  "lifecycle": {
    "rule": [
      {
        "action": {"type": "Delete"},
        "condition": {
          "age": 30,
          "isLive": false
        }
      }
    ]
  }
}
EOF

gsutil lifecycle set lifecycle.json gs://anyon-projects
```

#### 2.2 gcsfuse 설치 및 설정

```bash
# SSH 접속
ssh anyon@34.50.24.115

# gcsfuse 설치 (Ubuntu/Debian)
export GCSFUSE_REPO=gcsfuse-`lsb_release -c -s`
echo "deb https://packages.cloud.google.com/apt $GCSFUSE_REPO main" | \
  sudo tee /etc/apt/sources.list.d/gcsfuse.list
curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key add -

sudo apt-get update
sudo apt-get install -y gcsfuse

# 버전 확인
gcsfuse --version
```

#### 2.3 자동 마운트 설정

```bash
# systemd 서비스 생성
sudo tee /etc/systemd/system/gcsfuse-anyon.service > /dev/null << 'EOF'
[Unit]
Description=GCS FUSE mount for Anyon projects
After=network-online.target
Wants=network-online.target

[Service]
Type=forking
User=anyon
Group=anyon
ExecStart=/usr/bin/gcsfuse \
    --dir-mode 0755 \
    --file-mode 0644 \
    --implicit-dirs \
    --stat-cache-ttl 60s \
    --type-cache-ttl 60s \
    --kernel-list-cache-ttl-secs 60 \
    --max-conns-per-host 100 \
    --temp-dir /var/opt/anyon/tmp/gcsfuse \
    anyon-projects \
    /var/opt/anyon/projects-gcs
ExecStop=/bin/fusermount -u /var/opt/anyon/projects-gcs
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# 서비스 활성화
sudo systemctl daemon-reload
sudo systemctl enable gcsfuse-anyon.service
sudo systemctl start gcsfuse-anyon.service

# 상태 확인
sudo systemctl status gcsfuse-anyon.service

# 마운트 확인
df -h | grep anyon-projects
```

**gcsfuse 마운트 옵션 설명:**
- `--implicit-dirs`: 빈 디렉터리 자동 생성
- `--stat-cache-ttl 60s`: 파일 메타데이터 캐싱
- `--max-conns-per-host 100`: GCS 동시 연결 수
- `--temp-dir`: 로컬 캐시 디렉터리

#### 2.4 기존 프로젝트 마이그레이션

```bash
# 백업
sudo mv /var/opt/anyon/projects /var/opt/anyon/projects-backup

# 마운트 포인트 생성
sudo mkdir -p /var/opt/anyon/projects-gcs
sudo chown -R $USER:$USER /var/opt/anyon/projects-gcs

# 심볼릭 링크
sudo ln -s /var/opt/anyon/projects-gcs /var/opt/anyon/projects

# 마이그레이션 스크립트
cat > /tmp/migrate-to-gcs.sh << 'EOF'
#!/bin/bash
set -e

BACKUP_DIR="/var/opt/anyon/projects-backup"
GCS_DIR="/var/opt/anyon/projects-gcs"

if [ ! -d "$BACKUP_DIR" ]; then
    echo "No backup directory found."
    exit 0
fi

echo "Migrating projects to GCS..."

for project in "$BACKUP_DIR"/*; do
    if [ -d "$project" ]; then
        project_name=$(basename "$project")
        echo "Copying $project_name..."
        rsync -av --progress "$project/" "$GCS_DIR/$project_name/"
    fi
done

echo "Migration complete!"
EOF

chmod +x /tmp/migrate-to-gcs.sh
bash /tmp/migrate-to-gcs.sh
```

#### 2.5 테스트

```bash
# 1. GCS 마운트 확인
ssh anyon@34.50.24.115 "df -h | grep anyon-projects"

# 2. 파일 쓰기 테스트
ssh anyon@34.50.24.115 "echo 'test' > /var/opt/anyon/projects/test.txt"

# 3. GCS에서 확인
gsutil cat gs://anyon-projects/test.txt

# 4. 프로젝트 생성 테스트
curl -X POST http://34.50.24.115/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name": "gcs-test", "git_repo_path": ""}'

# 5. GCS 확인
gsutil ls gs://anyon-projects/gcs-test/

# 6. 성능 테스트
ssh anyon@34.50.24.115 "time dd if=/dev/zero of=/var/opt/anyon/projects/test-large bs=1M count=100"
```

---

### Phase 3: Worktree 컨테이너 내부 관리 (2일)

**목표:** Worktree를 프로젝트 디렉터리 안에서 관리

#### 3.1 목표 구조

**현재:**
```
/var/opt/anyon/
├─ projects/project-123/
│   ├─ .git/
│   └─ src/
└─ worktrees/
    ├─ task-1/
    └─ task-2/
```

**목표:**
```
/var/opt/anyon/projects/project-123/
├─ .git/
│   └─ worktrees/      ← Git 메타데이터
│       ├─ task-1/
│       └─ task-2/
├─ src/
└─ .worktrees/         ← 실제 작업 디렉터리
    ├─ task-1/
    └─ task-2/
```

#### 3.2 코드 수정

**`crates/services/src/services/container.rs`:**

```rust
impl LocalContainerService {
    /// Worktree 경로: 프로젝트 안에 생성
    fn get_worktree_path(&self, project: &Project, task_attempt: &TaskAttempt) -> PathBuf {
        let project_path = PathBuf::from(&project.git_repo_path);
        project_path
            .join(".worktrees")
            .join(format!("{}-{}", task_attempt.id, task_attempt.branch))
    }
}

async fn create(&self, task_attempt: &TaskAttempt) -> Result<ContainerRef, ContainerError> {
    let task = Task::find_by_id(&self.db.pool, task_attempt.task_id).await?
        .ok_or_else(|| ContainerError::Other(anyhow::anyhow!("Task not found")))?;

    let project = Project::find_by_id(&self.db.pool, task.project_id).await?
        .ok_or_else(|| ContainerError::Other(anyhow::anyhow!("Project not found")))?;

    let project_path = PathBuf::from(&project.git_repo_path);
    let worktree_path = self.get_worktree_path(&project, task_attempt);

    // .worktrees 디렉터리 생성
    if let Some(parent) = worktree_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Worktree 생성
    WorktreeManager::create_worktree(
        &project_path,
        &task_attempt.branch,
        &worktree_path,
        &task_attempt.target_branch,
        true,
    ).await?;

    Ok(ContainerRef::from(worktree_path))
}
```

#### 3.3 .gitignore 업데이트

```rust
// crates/server/src/routes/projects.rs

async fn create_project(...) -> Result<...> {
    // Git 저장소 초기화 후
    deployment.git().initialize_repo_with_main_branch(&path)?;

    // .gitignore에 worktrees 추가
    let gitignore_path = path.join(".gitignore");
    let gitignore_content = if gitignore_path.exists() {
        tokio::fs::read_to_string(&gitignore_path).await?
    } else {
        String::new()
    };

    if !gitignore_content.contains(".worktrees") {
        let updated = format!("{}\n# Anyon worktrees\n.worktrees/\n", gitignore_content);
        tokio::fs::write(&gitignore_path, updated).await?;
    }
}
```

#### 3.4 테스트

```bash
# 1. 프로젝트 생성
curl -X POST http://34.50.24.115/api/projects \
  -d '{"name": "worktree-test", "git_repo_path": ""}'

# 2. Task 실행
curl -X POST http://34.50.24.115/api/tasks \
  -d '{"project_id": "xxx", "description": "test"}'

# 3. 디렉터리 구조 확인
ssh anyon@34.50.24.115 "tree -L 3 /var/opt/anyon/projects/worktree-test/"

# 예상 출력:
# /var/opt/anyon/projects/worktree-test/
# ├── .git/
# │   └── worktrees/
# ├── .gitignore
# ├── .worktrees/
# │   └── task-xxx-feature/
# └── src/

# 4. GCS 확인
gsutil ls -r gs://anyon-projects/worktree-test/.worktrees/
```

---

### Phase 4: 모니터링 & 최적화 (5일)

**목표:** 성능 모니터링 및 캐싱 최적화

#### 4.1 Prometheus + Grafana 설치

```bash
# Prometheus
docker run -d \
  --name prometheus \
  -p 9090:9090 \
  -v /var/opt/anyon/monitoring/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus

# Grafana
docker run -d \
  --name grafana \
  -p 3001:3000 \
  -v /var/opt/anyon/monitoring/grafana:/var/lib/grafana \
  grafana/grafana
```

**`/var/opt/anyon/monitoring/prometheus.yml`:**

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'anyon-containers'
    static_configs:
      - targets: ['34.50.24.115:8080']

  - job_name: 'docker'
    static_configs:
      - targets: ['34.50.24.115:9323']

  - job_name: 'node'
    static_configs:
      - targets: ['34.50.24.115:9100']
```

#### 4.2 애플리케이션 메트릭 추가

```rust
// Cargo.toml
[dependencies]
prometheus = "0.13"

// crates/server/src/metrics.rs
use prometheus::{register_gauge_vec, register_histogram_vec, GaugeVec, HistogramVec};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONTAINER_COUNT: GaugeVec = register_gauge_vec!(
        "anyon_containers_total",
        "Number of running containers",
        &["status"]
    ).unwrap();

    pub static ref CONTAINER_EXEC_DURATION: HistogramVec = register_histogram_vec!(
        "anyon_container_exec_duration_seconds",
        "Container command execution duration",
        &["project_id", "status"],
        vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0]
    ).unwrap();

    pub static ref IDLE_CONTAINERS: GaugeVec = register_gauge_vec!(
        "anyon_idle_containers_total",
        "Number of idle containers by duration",
        &["idle_duration"]
    ).unwrap();
}

// 메트릭 엔드포인트
use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder};

pub fn metrics_routes() -> Router {
    Router::new().route("/metrics", get(metrics_handler))
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

#### 4.3 gcsfuse 캐싱 최적화

```bash
# systemd 서비스 수정
sudo systemctl edit --full gcsfuse-anyon.service

# ExecStart 최적화:
ExecStart=/usr/bin/gcsfuse \
    --dir-mode 0755 \
    --file-mode 0644 \
    --implicit-dirs \
    --stat-cache-ttl 300s \              # 5분으로 증가
    --type-cache-ttl 300s \
    --kernel-list-cache-ttl-secs 300 \
    --max-conns-per-host 200 \           # 연결 수 증가
    --temp-dir /var/opt/anyon/tmp/gcsfuse \
    --max-retry-sleep 30s \
    --stat-cache-capacity 100000 \       # 캐시 용량 증가
    --enable-storage-client-library \
    anyon-projects \
    /var/opt/anyon/projects-gcs

sudo systemctl daemon-reload
sudo systemctl restart gcsfuse-anyon.service
```

#### 4.4 부하 테스트

```bash
# load-test.sh
#!/bin/bash

ANYON_HOST="http://34.50.24.115"
NUM_PROJECTS=10
NUM_TASKS_PER_PROJECT=5

echo "Creating $NUM_PROJECTS projects..."
for i in $(seq 1 $NUM_PROJECTS); do
  PROJECT_NAME="load-test-project-$i"

  RESPONSE=$(curl -s -X POST "$ANYON_HOST/api/projects" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$PROJECT_NAME\", \"git_repo_path\": \"\"}")

  PROJECT_ID=$(echo $RESPONSE | jq -r '.data.id')
  echo "Created project $PROJECT_ID"

  for j in $(seq 1 $NUM_TASKS_PER_PROJECT); do
    echo "  Creating task $j..."
    curl -s -X POST "$ANYON_HOST/api/tasks" \
      -H "Content-Type: application/json" \
      -d "{\"project_id\": \"$PROJECT_ID\", \"description\": \"Load test $j\"}" \
      > /dev/null
  done
done

# 메트릭 수집
curl -s "$ANYON_HOST/metrics" > /tmp/anyon-metrics.txt
ssh anyon@34.50.24.115 "docker ps --format 'table {{.Names}}\t{{.Status}}'" \
  > /tmp/container-status.txt

echo "Load test complete!"
```

#### 4.5 Grafana 대시보드

```json
{
  "dashboard": {
    "title": "Anyon Container Metrics",
    "panels": [
      {
        "title": "Running Containers",
        "targets": [
          {"expr": "anyon_containers_total{status=\"running\"}"}
        ]
      },
      {
        "title": "Idle Containers",
        "targets": [
          {"expr": "anyon_idle_containers_total"}
        ]
      },
      {
        "title": "Execution Duration (P95)",
        "targets": [
          {"expr": "histogram_quantile(0.95, rate(anyon_container_exec_duration_seconds_bucket[5m]))"}
        ]
      }
    ]
  }
}
```

---

## 롤백 계획

### Phase 2 (GCS) 긴급 롤백

```bash
# 1. gcsfuse 언마운트
sudo systemctl stop gcsfuse-anyon.service

# 2. 기존 디렉터리 복원
sudo rm /var/opt/anyon/projects
sudo mv /var/opt/anyon/projects-backup /var/opt/anyon/projects

# 3. 서비스 재시작
sudo systemctl restart anyon-server

# 4. 확인
curl http://34.50.24.115/api/projects
```

### Phase 1 (컨테이너) 롤백

```bash
# Git 브랜치로 되돌리기
git checkout main
git branch -D feat/replit-style-containers

# 재빌드 및 배포
cargo build --release --features cloud
sudo systemctl restart anyon-server
```

---

## 체크리스트

### Phase 1 완료 기준
- [ ] 프로젝트당 1개 컨테이너 생성 확인
- [ ] 여러 Task가 같은 컨테이너 재사용 확인
- [ ] Idle 타임아웃 동작 확인 (30분)
- [ ] 메모리 누수 없음 (24시간 모니터링)
- [ ] 기존 기능 정상 동작 (회귀 테스트)
- [ ] 프로젝트 삭제 시 컨테이너 정리 확인

### Phase 2 완료 기준
- [ ] GCS 버킷 생성 및 권한 설정
- [ ] gcsfuse 마운트 정상 동작
- [ ] 파일 읽기/쓰기 정상
- [ ] 기존 프로젝트 마이그레이션 완료
- [ ] 부팅 시 자동 마운트 확인
- [ ] GCS 동기화 확인

### Phase 3 완료 기준
- [ ] Worktree가 프로젝트 디렉터리 안에 생성됨
- [ ] GCS에 Worktree 저장 확인
- [ ] Git 명령어 정상 동작
- [ ] Worktree 정리 로직 정상 동작
- [ ] .gitignore 업데이트 확인

### Phase 4 완료 기준
- [ ] Prometheus + Grafana 설치 완료
- [ ] 메트릭 수집 정상 동작
- [ ] 대시보드 구성 완료
- [ ] 부하 테스트 통과 (10 projects, 50 tasks)
- [ ] gcsfuse 캐싱 최적화 확인

---

## 예상 효과

### 정량적 개선

| 지표 | 현재 | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|------|------|---------|---------|---------|---------|
| Task 시작 시간 | ~10초 | ~2초 | ~2초 | ~2초 | ~1초 |
| 환경 초기화 | 매번 | 프로젝트 1회 | 프로젝트 1회 | 프로젝트 1회 | 프로젝트 1회 |
| 데이터 손실 위험 | 높음 | 중간 | 낮음 | 낮음 | 낮음 |
| VM 독립성 | 없음 | 없음 | 있음 | 있음 | 있음 |
| 멀티테넌시 | 불가능 | 가능 | 가능 | 가능 | 가능 |
| 모니터링 | 없음 | 없음 | 없음 | 없음 | 완전 |

### 정성적 개선

**사용자 경험:**
- ✅ Replit과 동일한 빠른 실행 속도
- ✅ 프로젝트 환경이 계속 유지됨
- ✅ 데이터 손실 걱정 없음

**운영 효율:**
- ✅ VM 스케일링 용이
- ✅ 장애 복구 간단
- ✅ 모니터링으로 문제 조기 발견

**비즈니스 가치:**
- ✅ 멀티테넌시로 SaaS 가능
- ✅ 안정적인 서비스 제공
- ✅ 확장 가능한 아키텍처

---

## 타임라인

| 주차 | Phase | 주요 작업 | 예상 소요 | 상태 |
|------|-------|-----------|-----------|------|
| Week 1 Day 1-3 | Phase 1 | 컨테이너 수명 변경 코드 수정 | 3일 | 🔴 대기 |
| Week 1 Day 4-5 | Phase 1 | 테스트 및 디버깅 | 2일 | 🔴 대기 |
| Week 2 Day 1-2 | Phase 2 | GCS 설정 및 gcsfuse 구성 | 2일 | 🔴 대기 |
| Week 2 Day 3-4 | Phase 2 | 마이그레이션 및 테스트 | 2일 | 🔴 대기 |
| Week 2 Day 5-Week 3 Day 1 | Phase 3 | Worktree 최적화 | 2일 | 🔴 대기 |
| Week 3 Day 2-4 | Phase 4 | 모니터링 구축 | 3일 | 🔴 대기 |
| Week 3 Day 5 | Phase 4 | 부하 테스트 | 1일 | 🔴 대기 |
| Week 4 | Launch | 최종 검증 및 배포 | 5일 | 🔴 대기 |

---

## 유용한 명령어

### VM 관리

```bash
# VM 접속
ssh anyon@34.50.24.115

# 서비스 로그
sudo journalctl -u anyon-server -f
sudo journalctl -u gcsfuse-anyon -f

# 컨테이너 상태
docker ps -a | grep project-
docker stats

# 디스크 사용량
df -h
du -sh /var/opt/anyon/*
```

### GCS 관리

```bash
# 버킷 내용 확인
gsutil ls -r gs://anyon-projects/

# 파일 업로드/다운로드
gsutil cp /local/file gs://anyon-projects/
gsutil cp gs://anyon-projects/file /local/

# 동기화
gsutil -m rsync -r /local/dir gs://anyon-projects/dir/
```

### 성능 모니터링

```bash
# 시스템 리소스
htop
iotop

# 네트워크
ping -c 5 storage.googleapis.com
traceroute storage.googleapis.com

# gcsfuse 캐시
du -sh /var/opt/anyon/tmp/gcsfuse
```

---

## 트러블슈팅

### gcsfuse 마운트 실패

```bash
# 로그 확인
sudo journalctl -u gcsfuse-anyon -n 50

# 수동 마운트 시도
gcsfuse --debug_fuse --debug_gcs anyon-projects /var/opt/anyon/projects-gcs

# 권한 확인
gcloud auth list
gcloud projects list
```

### 컨테이너 중복 생성

```bash
# Provisioned 맵 디버깅 (코드에 로그 추가)
# container_id 중복 확인
docker ps --format '{{.Names}}' | sort | uniq -d

# 수동 정리
docker stop $(docker ps -q --filter "name=project-")
docker rm $(docker ps -aq --filter "name=project-")
```

### GCS 성능 저하

```bash
# gcsfuse 통계
cat /sys/fs/fuse/connections/*/congestion_threshold

# 캐시 히트율 확인
# (메트릭에서 확인)

# 마운트 옵션 재설정
sudo systemctl restart gcsfuse-anyon
```

---

## 참고 자료

### Replit 아키텍처
- [Replit Storage: The Next Generation](https://blog.replit.com/replit-storage-the-next-generation)
- [Regional Goval Project](https://blog.replit.com/regional-goval)
- [Killing Containers at Scale](https://blog.replit.com/killing-containers-at-scale)

### GCS & gcsfuse
- [gcsfuse Documentation](https://github.com/GoogleCloudPlatform/gcsfuse)
- [GCS Best Practices](https://cloud.google.com/storage/docs/best-practices)

### Docker
- [Docker Container Management](https://docs.docker.com/engine/reference/commandline/container/)
- [Docker Resource Constraints](https://docs.docker.com/config/containers/resource_constraints/)

---

**문서 종료**

이 계획서에 대한 질문이나 수정 사항이 있으면 팀에 문의하세요.
