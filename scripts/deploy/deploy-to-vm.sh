#!/bin/bash

set -euo pipefail

VM_IP="${1:-}"
if [[ -z "$VM_IP" ]]; then
    echo "Usage: ./deploy-to-vm.sh <VM_IP>" >&2
    exit 1
fi

if ! command -v pnpm >/dev/null 2>&1; then
    echo "pnpm이 설치되어 있지 않습니다. https://pnpm.io/installation 을 참고하세요." >&2
    exit 1
fi

ZONE="asia-northeast3-a"
INSTANCE_NAME="anyon"
REMOTE_DIR="~/anyon"

cat <<MSG
🚀 Starting deployment to $VM_IP...
MSG

# 1. 프론트엔드 빌드
echo "🎨 Building frontend..."
(
    cd frontend
    pnpm install --frozen-lockfile || pnpm install
    pnpm run build
)

# 2. 배포 파일 준비
echo "📂 Preparing deployment files..."
rm -rf deploy_temp
mkdir -p deploy_temp/frontend deploy_temp/migrations deploy_temp/data deploy_temp/src
cp -r frontend/dist deploy_temp/frontend
cp -r crates deploy_temp/src/
cp Cargo.toml deploy_temp/src/ 2>/dev/null || true
cp Cargo.lock deploy_temp/src/ 2>/dev/null || true
cp -r crates/db/migrations deploy_temp/migrations
if [[ -f scripts/deploy/.env.production.template ]]; then
    cp scripts/deploy/.env.production.template deploy_temp/.env.production.template
fi

# 3. VM에 파일 전송
echo "📤 Uploading files to VM..."
gcloud compute ssh "$INSTANCE_NAME" --zone="$ZONE" --command='mkdir -p ~/anyon' || true
gcloud compute scp --recurse deploy_temp/* \
    "$INSTANCE_NAME:~/anyon/" \
    --zone="$ZONE"

# 4. VM에서 빌드 및 실행
echo "⚙️ Setting up on VM..."
gcloud compute ssh "$INSTANCE_NAME" --zone="$ZONE" --command='bash -s' <<'REMOTE'
set -euo pipefail
cd ~/anyon

# Rust 설치 확인 및 설치
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 필요한 시스템 패키지 설치
if ! dpkg -l | grep -q libssl-dev; then
    echo "📦 Installing system dependencies..."
    sudo apt-get update
    sudo apt-get install -y libssl-dev pkg-config build-essential
fi

export ANYON_CLOUD_BASE_DIR="$HOME/anyon"
export ANYON_ASSET_DIR="$HOME/anyon/data"
export ANYON_TEMP_DIR="$HOME/anyon/tmp"
export ANYON_WORKTREE_DIR="$HOME/anyon/worktrees"
export ANYON_DATABASE_FILE="$HOME/anyon/data/anyon.db"
export ANYON_DOCKER_USER="$USER"
export ANYON_LOG_FILE="$HOME/anyon/logs/server.log"
mkdir -p "$ANYON_ASSET_DIR" "$ANYON_TEMP_DIR" "$ANYON_WORKTREE_DIR" "$(dirname "$ANYON_LOG_FILE")"

# 소스 코드가 있으면 빌드
if [[ -d src/crates ]]; then
    echo "🔨 Building server binary on VM..."
    cd src
    source "$HOME/.cargo/env" 2>/dev/null || true
    cargo build --release --features cloud || {
        echo "⚠️ Build failed, trying with default features..."
        cargo build --release
    }
    cp target/release/server ../server
    cd ..
fi

# Dockerfile 생성
if [[ ! -f Dockerfile ]]; then
cat > Dockerfile <<'DOCKER'
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    git \
    vim \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://install.claude.ai/linux | bash

WORKDIR /workspace
CMD ["/bin/bash"]
DOCKER
fi

sudo docker build -t anyon-claude:latest .

# 서버 실행
export DATABASE_URL="sqlite://$ANYON_DATABASE_FILE"
export BACKEND_PORT=3000
export HOST=0.0.0.0
pkill -f ./server || true
chmod +x server
nohup ./server > "$ANYON_LOG_FILE" 2>&1 &
sleep 3

if ps aux | grep -q "[.]/server"; then
    echo "✅ Server started on port 3000"
    echo "📊 Check logs: tail -f $ANYON_LOG_FILE"
else
    echo "❌ Server failed to start. Check logs:"
    tail -20 "$ANYON_LOG_FILE"
    exit 1
fi
REMOTE

# 6. 로컬 정리
rm -rf deploy_temp

echo ""
echo "🎉 Deployment complete!"
echo "🌐 Access Anyon at: http://$VM_IP:3000"
echo ""
echo "📝 Useful commands:"
echo "  - Check status:  gcloud compute ssh $INSTANCE_NAME --zone=$ZONE --command='ps aux | grep server'"
echo "  - View logs:     gcloud compute ssh $INSTANCE_NAME --zone=$ZONE --command='tail -f ~/anyon/logs/server.log'"
echo "  - Stop server:   gcloud compute ssh $INSTANCE_NAME --zone=$ZONE --command='pkill -f server'"
