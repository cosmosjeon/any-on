# Phase 2 Deployment Checklist

## Pre-Deployment
- [ ] Phase 0-1 completed
- [ ] `cargo build --release` succeeds locally
- [ ] `pnpm run dev` works locally with Docker
- [ ] GCP account ready with billing
- [ ] `gcloud` CLI installed and authenticated
- [ ] GCP project ID obtained

## Deployment Steps
- [ ] Run `create-gcp-vm.sh`
- [ ] Note down the VM external IP: _______________
- [ ] Run `deploy-to-vm.sh <VM_IP>`
- [ ] Run `health-check.sh <VM_IP>`
- [ ] Access http://<VM_IP>:3000 in browser
- [ ] Test creating a Project
- [ ] Test creating a Task
- [ ] Test Task execution (Docker container runs)
- [ ] Test WebSocket logs streaming

## Verification
- [ ] Can access from my computer
- [ ] Can access from different network (mobile hotspot)
- [ ] Task execution creates Docker container on VM
- [ ] Logs stream in real-time
- [ ] No errors in server logs

## Success Criteria
✅ All checks above passed
✅ Application accessible from anywhere
✅ Docker containers run on GCP VM
✅ Same functionality as localhost

## Rollback Plan (if something goes wrong)
1. Keep localhost version running
2. Debug VM issues using logs
3. If unfixable, delete VM and retry
4. Worst case: continue using localhost

## Cost Management
- [ ] Set up billing alerts in GCP
- [ ] Remember to stop/delete VM when testing is done
- [ ] Estimated cost: ~$5/day for testing

## Post-Deployment
- [ ] Document the VM IP for team
- [ ] Share access instructions
- [ ] Plan Phase 3 (GitHub integration)
