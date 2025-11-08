#!/bin/bash

set -euo pipefail

# GCP 프로젝트 설정
PROJECT_ID="${GCP_PROJECT_ID:-your-project-id}"
ZONE="asia-northeast3-a"  # 서울
INSTANCE_NAME="anyon"
MACHINE_TYPE="n2-standard-4"
BOOT_DISK_SIZE="100GB"

if [[ "$PROJECT_ID" == "your-project-id" ]]; then
    echo "[경고] GCP_PROJECT_ID 환경변수를 설정하세요." >&2
fi

STARTUP_SCRIPT=$(cat <<'SCRIPT'
#!/bin/bash
set -euo pipefail
apt-get update
apt-get install -y docker.io git curl build-essential
systemctl enable docker
systemctl start docker
DEFAULT_USER=$(getent passwd 1000 | cut -d: -f1 || echo "ubuntu")
if id "$DEFAULT_USER" &>/dev/null; then
    usermod -aG docker "$DEFAULT_USER"
fi
SCRIPT
)

# VM 생성
gcloud compute instances create "$INSTANCE_NAME" \
    --project="$PROJECT_ID" \
    --zone="$ZONE" \
    --machine-type="$MACHINE_TYPE" \
    --boot-disk-size="$BOOT_DISK_SIZE" \
    --boot-disk-type=pd-standard \
    --image-family=ubuntu-2204-lts \
    --image-project=ubuntu-os-cloud \
    --tags=http-server,https-server \
    --metadata=startup-script="$STARTUP_SCRIPT"

# 방화벽 규칙 생성 (3000번 포트)
if ! gcloud compute firewall-rules describe allow-anyon --project="$PROJECT_ID" >/dev/null 2>&1; then
    gcloud compute firewall-rules create allow-anyon \
        --project="$PROJECT_ID" \
        --allow=tcp:3000 \
        --source-ranges=0.0.0.0/0 \
        --target-tags=http-server
else
    echo "방화벽 규칙 allow-anyon 이 이미 존재합니다."
fi

# VM IP 주소 출력
echo "VM 생성 완료!"
gcloud compute instances describe "$INSTANCE_NAME" \
    --project="$PROJECT_ID" \
    --zone="$ZONE" \
    --format='get(networkInterfaces[0].accessConfigs[0].natIP)'
