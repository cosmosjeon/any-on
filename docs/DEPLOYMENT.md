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
