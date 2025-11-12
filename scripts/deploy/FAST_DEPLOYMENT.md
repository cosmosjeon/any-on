# 빠른 클라우드 배포 가이드

## 문제점

현재 배포 스크립트는 **VM에서 빌드**하기 때문에 매우 느립니다:

1. **VM에서 빌드**: `cargo build --release`는 수 분~수십 분 소요
2. **개발 흐름 단절**: 코드 변경마다 전체 빌드 필요
3. **리소스 낭비**: VM CPU/메모리를 빌드에 사용

## 해결 방법

### 방법 1: 빠른 배포 스크립트 사용 (권장) ⚡

로컬에서 빌드하고 바이너리만 업로드/재시작:

```bash
# 같은 VM에서 개발 중일 때 (디버그 빌드, 매우 빠름)
./scripts/deploy/deploy-fast.sh local debug

# 다른 VM으로 배포할 때 (gcloud 필요)
./scripts/deploy/deploy-fast.sh <VM_IP> debug

# 프로덕션용 (느리지만 최적화됨, release 빌드)
./scripts/deploy/deploy-fast.sh <VM_IP> release
```

**장점:**
- ⚡ **10-100배 빠름**: 바이너리만 업로드 (수 초)
- 🔄 **개발 흐름 유지**: 로컬에서 빌드하고 즉시 배포
- 💻 **로컬 리소스 활용**: 더 빠른 머신에서 빌드

### 방법 2: 개발 중에는 로컬 모드 사용

클라우드 배포가 필요 없을 때는 로컬에서 개발:

```bash
# 로컬에서 개발 (가장 빠름)
pnpm run dev
```

### 방법 3: 증분 빌드 활용

로컬에서 빌드할 때 증분 컴파일 사용:

```bash
# Cargo.toml에 추가 (이미 기본값이지만 확인)
[profile.dev]
incremental = true

[profile.release]
incremental = true
```

### 방법 4: Docker 빌드 캐시 활용

VM에서 빌드할 때도 캐시 활용:

```bash
# VM에서 빌드 시 캐시 디렉터리 유지
export CARGO_TARGET_DIR="$HOME/.cargo/target"
cargo build --release --features cloud
```

## 성능 비교

| 방법 | 빌드 시간 | 업로드 시간 | 총 시간 |
|------|----------|------------|--------|
| VM에서 빌드 (현재) | 5-30분 | 0초 | 5-30분 |
| 로컬 빌드 + 업로드 (개선) | 1-5분 | 5-10초 | 1-5분 |
| Debug 빌드 + 업로드 | 10-30초 | 5-10초 | 15-40초 |

## 사용 예시

### 개발 중 빠른 반복

```bash
# 1. 코드 수정
vim crates/server/src/routes/projects.rs

# 2. 빠른 배포 (debug 모드)
# VM 내부라면
./scripts/deploy/deploy-fast.sh local debug

# 다른 VM으로 보내면
./scripts/deploy/deploy-fast.sh 34.50.24.115 debug

# 3. 테스트
curl http://34.50.24.115:3000/api/health
```

### 프로덕션 배포

```bash
# 최적화된 빌드로 배포
./scripts/deploy/deploy-fast.sh 34.50.24.115 release
```

## 추가 최적화 팁

1. **로컬 빌드 캐시 활용**
   ```bash
   # Cargo 빌드 캐시 유지
   export CARGO_TARGET_DIR="$HOME/.cargo/target"
   ```

2. **병렬 빌드**
   ```bash
   # Cargo.toml에 추가
   [build]
   jobs = 8  # CPU 코어 수에 맞게 조정
   ```

3. **링커 최적화** (선택사항)
   ```bash
   # mold 링커 사용 (Linux)
   cargo install cargo-mold
   cargo mold build --release --features cloud
   ```

4. **개발 중에는 debug 빌드**
   - Debug 빌드는 10-100배 빠름
   - 개발 중에는 충분히 빠름

