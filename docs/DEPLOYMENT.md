# Anyon - GCP Deployment Guide

## Phase 2: Google Cloud VM Deployment

### Prerequisites
1. Google Cloud Platform account with billing enabled
2. `gcloud` CLI installed and authenticated
3. Local build successful (Phase 0-1 completed)

### Step 1: Create GCP VM

```bash
# Set your GCP project ID
export GCP_PROJECT_ID=your-project-id

# Run the creation script
./scripts/deploy/create-gcp-vm.sh
```

This will:

- Create a VM in Seoul region (asia-northeast3-a)
- Install Docker and dependencies
- Configure firewall rules
- Output the VM's external IP address

Save the VM IP address! You'll need it for the next steps.

### Step 2: Deploy to VM

```bash
# Deploy using the VM IP from Step 1
./scripts/deploy/deploy-to-vm.sh <VM_IP>
```

This will:

- Build the release binary (`cargo build --release --features cloud`)
- Build the frontend bundle (`pnpm install && pnpm run build` inside `frontend/`)
- Upload files to the VM
- Ensure the Claude-enabled Docker image exists
- Restart the Anyon server on port 3000

> The deploy script also exports the following environment variables on the VM so CloudDeployment uses the correct paths:
> `ANYON_CLOUD_BASE_DIR=~/anyon`, `ANYON_ASSET_DIR=~/anyon/data`, `ANYON_TEMP_DIR=~/anyon/tmp`, `ANYON_WORKTREE_DIR=~/anyon/worktrees`, `ANYON_DATABASE_FILE=~/anyon/data/anyon.db`, `ANYON_LOG_FILE=~/anyon/logs/server.log`.

### Step 2.1: Docker Runtime 이미지 준비

CloudContainerService는 Docker 컨테이너 안에서 실행되므로 Claude Code CLI와 GitHub CLI가 포함된 이미지를 준비해야 합니다.

1. 이미지 빌드
   ```bash
   ANYON_CLOUD_IMAGE=registry.example.com/anyon/claude-runtime:v0.1.0 \
   PUSH_IMAGE=true \
   ./scripts/docker/build-claude-runtime.sh
   ```
2. 배포 VM 또는 환경 변수에 이미지 명시
   ```bash
   export ANYON_CLOUD_CONTAINER_IMAGE=registry.example.com/anyon/claude-runtime:v0.1.0
   ```
3. 이미지 구성 요소
   - Node.js 20.x
   - `@anthropic-ai/claude-code` CLI (`CLAUDE_CODE_VERSION` 빌드 인자로 조절 가능)
   - `@github/cli`
   - 비루트 사용자 `anyon`, 작업 디렉터리 `/workspace`, 비밀 마운트 `/tmp/anyon-secrets`

> GitHub Actions 예시는 `.github/workflows/docker-image.yml`을 참고하세요. `DOCKER_USERNAME`/`DOCKER_PASSWORD`/`DOCKER_REGISTRY` 시크릿을 설정하면 push까지 자동화할 수 있습니다.

### Step 2.2: Secret 디렉터리 정책

- 호스트: `/tmp/anyon/cloud-secrets/<task_attempt_id>` – GitHub/Claude 토큰이 암호화 해제된 상태로 잠시 저장되는 위치입니다.
- 컨테이너: `/tmp/anyon-secrets` – 위 디렉터리를 bind mount하여 `CLAUDE_CONFIG_PATH`, `GIT_CONFIG_GLOBAL`, `GITHUB_TOKEN`, `GH_TOKEN` 환경 변수를 노출합니다.
- 실행이 끝나거나 컨테이너가 삭제되면 해당 디렉터리를 제거합니다. 남아 있지 않은지 주기적으로 검증하세요 (`docs/runbooks/cloud-runtime-checklist.md` 참조).
 - 실행이 끝나거나 컨테이너가 삭제되면 해당 디렉터리를 제거합니다. 남아 있지 않은지 주기적으로 검증하세요 (참고: [`docs/runbooks/cloud-runtime-checklist.md`](./runbooks/cloud-runtime-checklist.md)).

필수 환경 변수 요약:

| 변수 | 설명 |
| --- | --- |
| `ANYON_CLOUD_CONTAINER_IMAGE` | Cloud 런타임에서 사용할 컨테이너 이미지 (예: `registry.example.com/anyon/claude-runtime:v0.1.0`). |
| `ANYON_SECRET_KEY` | SecretStore 암호화 키 (base64 32 bytes). |
| `ANYON_TEMP_DIR` | secret 디렉터리 루트(`/tmp/anyon/cloud-secrets`)를 포함한 임시 디렉터리. |
| `ANYON_DOCKER_USER` | 컨테이너 실행 시 권한을 맞추고 싶다면 설정 (기본 `ubuntu`). |

### Step 3: Verify Deployment

```bash
# Run health checks
./scripts/deploy/health-check.sh <VM_IP>
```

### Step 4: Access Your Application

Open your browser and go to:

```
http://<VM_IP>:3000
```

You should see the Anyon interface!

### Troubleshooting

**Server not starting:**

```bash
gcloud compute ssh anyon --zone=asia-northeast3-a \
  --command='tail -100 ~/anyon/server.log'
```

**Port not accessible:**

```bash
gcloud compute firewall-rules list | grep anyon
```

**Docker issues:**

```bash
gcloud compute ssh anyon --zone=asia-northeast3-a \
  --command='sudo systemctl status docker'
```

### Useful Commands

Stop the server:

```bash
gcloud compute ssh anyon --zone=asia-northeast3-a \
  --command='pkill -f server'
```

Restart the server:

```bash
gcloud compute ssh anyon --zone=asia-northeast3-a \
  --command='cd ~/anyon && nohup ./server > server.log 2>&1 &'
```

Delete the VM (when done testing):

```bash
gcloud compute instances delete anyon --zone=asia-northeast3-a
```

### Cost Estimation

n2-standard-4 VM in Seoul:

- ~$120/month if running 24/7
- ~$5/day for testing

Remember to delete the VM when not in use!

### Next Steps

After Phase 2 is complete:

- Phase 3: GitHub integration for cloning repositories
- Phase 4: Claude credentials secure storage
