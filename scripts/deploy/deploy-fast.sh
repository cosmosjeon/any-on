#!/bin/bash
# 빠른 배포 스크립트: 로컬에서 빌드하고 바이너리만 업로드

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

VM_IP="${1:-local}"
BUILD_MODE="${2:-release}"  # release 또는 debug

if [[ "$BUILD_MODE" != "release" && "$BUILD_MODE" != "debug" ]]; then
    echo "Usage: ./scripts/deploy/deploy-fast.sh [VM_IP|local] [release|debug]" >&2
    exit 1
fi

if [[ "$VM_IP" == "local" || "$VM_IP" == "self" || "$VM_IP" == "localhost" || "$VM_IP" == "127.0.0.1" ]] || ! command -v gcloud >/dev/null 2>&1; then
    DEPLOY_MODE="local"
else
    DEPLOY_MODE="remote"
fi

ZONE="${FAST_DEPLOY_GCLOUD_ZONE:-asia-northeast3-a}"
INSTANCE_NAME="${FAST_DEPLOY_INSTANCE_NAME:-anyon}"

cat <<MSG
🚀 빠른 배포 시작 (빌드 모드: $BUILD_MODE)...
MSG

# 1. 로컬에서 백엔드 빌드
echo "🔨 로컬에서 백엔드 빌드 중..."
if [[ "$BUILD_MODE" == "debug" ]]; then
    echo "   (개발 모드: 빠르지만 최적화 안 됨)"
    cargo build --features cloud --bin server
    BINARY_PATH="target/debug/server"
else
    echo "   (프로덕션 모드: 느리지만 최적화됨)"
    cargo build --release --features cloud --bin server
    BINARY_PATH="target/release/server"
fi

if [[ ! -f "$BINARY_PATH" ]]; then
    echo "❌ 빌드 실패: $BINARY_PATH를 찾을 수 없습니다" >&2
    exit 1
fi

echo "✅ 빌드 완료: $BINARY_PATH"

# 2. 프론트엔드 빌드 (변경된 경우만)
echo "🎨 프론트엔드 빌드 확인 중..."
(
    cd frontend
    if [[ ! -d "dist" ]] || [[ "frontend" -nt "frontend/dist" ]]; then
        echo "   프론트엔드 빌드 중..."
        pnpm install --frozen-lockfile || pnpm install
        pnpm run build
    else
        echo "   프론트엔드 빌드 스킵 (변경 없음)"
    fi
)

# 3. 배포 대상에 전달
if [[ "$DEPLOY_MODE" == "local" ]]; then
    echo "📂 로컬 배포 디렉터리 준비 중..."
    DEPLOY_ROOT="${FAST_DEPLOY_LOCAL_ROOT:-$PROJECT_ROOT}"
    mkdir -p "$DEPLOY_ROOT"

    cp "$BINARY_PATH" "$DEPLOY_ROOT/server"

    if [[ -d "frontend/dist" ]]; then
        mkdir -p "$DEPLOY_ROOT/frontend"
        if command -v rsync >/dev/null 2>&1; then
            rsync -a --delete frontend/dist/ "$DEPLOY_ROOT/frontend/dist/"
        else
            rm -rf "$DEPLOY_ROOT/frontend/dist"
            mkdir -p "$DEPLOY_ROOT/frontend/dist"
            cp -a frontend/dist/. "$DEPLOY_ROOT/frontend/dist/"
        fi
    fi

    if [[ -f "$DEPLOY_ROOT/.env.cloud" ]]; then
        echo "🧾 .env.cloud 로드"
        # shellcheck disable=SC2046,SC2002
        export $(cat "$DEPLOY_ROOT/.env.cloud" | grep -v '^#' | xargs) || true
    fi

    export ANYON_CLOUD_BASE_DIR="${ANYON_CLOUD_BASE_DIR:-$DEPLOY_ROOT}"
    export ANYON_ASSET_DIR="${ANYON_ASSET_DIR:-$ANYON_CLOUD_BASE_DIR/data}"
    export ANYON_TEMP_DIR="${ANYON_TEMP_DIR:-$ANYON_CLOUD_BASE_DIR/tmp}"
    export ANYON_WORKTREE_DIR="${ANYON_WORKTREE_DIR:-$ANYON_CLOUD_BASE_DIR/worktrees}"
    export ANYON_WORKSPACE_DIR="${ANYON_WORKSPACE_DIR:-$ANYON_CLOUD_BASE_DIR/workspace}"
    export ANYON_DATABASE_FILE="${ANYON_DATABASE_FILE:-$ANYON_CLOUD_BASE_DIR/data/anyon.db}"
    export ANYON_LOG_FILE="${ANYON_LOG_FILE:-$ANYON_CLOUD_BASE_DIR/logs/server.log}"
    export BACKEND_PORT="${BACKEND_PORT:-3000}"
    export HOST="${HOST:-0.0.0.0}"
    export DATABASE_URL="sqlite://$ANYON_DATABASE_FILE"

    mkdir -p "$ANYON_ASSET_DIR" "$ANYON_TEMP_DIR" "$ANYON_WORKTREE_DIR" "$ANYON_WORKSPACE_DIR" "$(dirname "$ANYON_LOG_FILE")"

    if [[ ! -f "$ANYON_ASSET_DIR/.secret_key" ]]; then
        openssl rand -base64 32 > "$ANYON_ASSET_DIR/.secret_key"
        chmod 600 "$ANYON_ASSET_DIR/.secret_key"
    fi
    export ANYON_SECRET_KEY=$(cat "$ANYON_ASSET_DIR/.secret_key")

    echo "🛑 기존 서버 중지 중..."
    pkill -f "$DEPLOY_ROOT/server" || true
    sleep 1

    echo "🚀 새 서버 시작..."
    chmod +x "$DEPLOY_ROOT/server"
    nohup "$DEPLOY_ROOT/server" > "$ANYON_LOG_FILE" 2>&1 &
    sleep 2

    if pgrep -f "$DEPLOY_ROOT/server" >/dev/null 2>&1; then
        echo "✅ 서버 시작 성공 (로컬)"
        echo "📊 로그 확인: tail -f $ANYON_LOG_FILE"
    else
        echo "❌ 서버 시작 실패. 로그 확인:"
        tail -20 "$ANYON_LOG_FILE"
        exit 1
    fi
else
    echo "📤 VM에 파일 업로드 중..."
    gcloud compute ssh "$INSTANCE_NAME" --zone="$ZONE" --command='mkdir -p ~/anyon' || true

    gcloud compute scp "$BINARY_PATH" "$INSTANCE_NAME:~/anyon/server" --zone="$ZONE"

    if [[ -d "frontend/dist" ]]; then
        gcloud compute ssh "$INSTANCE_NAME" --zone="$ZONE" --command='mkdir -p ~/anyon/frontend' || true
        gcloud compute scp --recurse frontend/dist "$INSTANCE_NAME:~/anyon/frontend/" --zone="$ZONE"
    fi

    echo "⚙️ VM에서 서버 재시작 중..."
    gcloud compute ssh "$INSTANCE_NAME" --zone="$ZONE" --command='bash -s' <<'REMOTE'
set -eo pipefail
cd ~/anyon

# 환경 변수 설정
export ANYON_CLOUD_BASE_DIR="$HOME/anyon"
export ANYON_ASSET_DIR="$HOME/anyon/data"
export ANYON_TEMP_DIR="$HOME/anyon/tmp"
export ANYON_WORKTREE_DIR="$HOME/anyon/worktrees"
export ANYON_DATABASE_FILE="$HOME/anyon/data/anyon.db"
export ANYON_DOCKER_USER="$USER"
export ANYON_LOG_FILE="$HOME/anyon/logs/server.log"
export DATABASE_URL="sqlite://$ANYON_DATABASE_FILE"
export BACKEND_PORT=3000
export HOST=0.0.0.0

mkdir -p "$ANYON_ASSET_DIR" "$ANYON_TEMP_DIR" "$ANYON_WORKTREE_DIR" "$(dirname "$ANYON_LOG_FILE")"

# ANYON_SECRET_KEY 확인
if [[ ! -f "$ANYON_ASSET_DIR/.secret_key" ]]; then
    openssl rand -base64 32 > "$ANYON_ASSET_DIR/.secret_key"
    chmod 600 "$ANYON_ASSET_DIR/.secret_key"
fi
export ANYON_SECRET_KEY=$(cat "$ANYON_ASSET_DIR/.secret_key")

# 기존 서버 중지
pkill -f "./server" || true
sleep 1

# 새 서버 시작
chmod +x server
nohup ./server > "$ANYON_LOG_FILE" 2>&1 &
sleep 2

if ps aux | grep -q "[.]/server"; then
    echo "✅ 서버 시작 성공"
    echo "📊 로그 확인: tail -f $ANYON_LOG_FILE"
else
    echo "❌ 서버 시작 실패. 로그 확인:"
    tail -20 "$ANYON_LOG_FILE"
    exit 1
fi
REMOTE
fi

echo ""
echo "🎉 빠른 배포 완료!"
if [[ "$DEPLOY_MODE" == "local" ]]; then
    echo "🌐 접속: http://${HOST:-0.0.0.0}:${BACKEND_PORT:-3000}"
else
    echo "🌐 접속: http://$VM_IP:3000"
fi
echo ""
echo "💡 팁:"
echo "   - 개발 중에는: ./scripts/deploy/deploy-fast.sh local debug  (더 빠름)"
echo "   - 프로덕션에는: ./scripts/deploy/deploy-fast.sh ${VM_IP:-<VM_IP>} release  (최적화됨)"

