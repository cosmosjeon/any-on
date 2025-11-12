# 빠른 개발 가이드 (Quick Development Guide)

VM에서 빠르게 개발하고 테스트하는 방법입니다.

## 🚀 빠른 재시작 (30~40초)

```bash
./scripts/quick-dev.sh
```

**특징:**
- Debug 빌드 사용 (최적화 안 함 → 빠른 컴파일)
- 서버 자동 중지 + 재시작
- Health check 포함
- Release 빌드 대비 **3~5배 빠름**

**빌드 시간 비교:**
- Release 빌드: 3~6분
- Debug 빌드: **30~40초** ⚡

## 📝 로그 확인

```bash
# 실시간 로그
tail -f /tmp/anyon-server.log

# 최근 100줄
tail -100 /tmp/anyon-server.log

# 에러만 보기
grep -i error /tmp/anyon-server.log | tail -20
```

## 🛑 서버 중지

```bash
# 스크립트가 출력한 PID 사용
kill <PID>

# 또는 프로세스 찾아서 중지
pkill -f "target/debug/server"
```

## ⚙️ 주의사항

### Debug vs Release

**Debug 빌드 (개발용):**
- ✅ 빠른 컴파일
- ✅ 디버그 심볼 포함
- ❌ 느린 실행 속도
- ❌ 큰 바이너리 크기

**Release 빌드 (프로덕션):**
- ❌ 느린 컴파일 (3~6분)
- ✅ 빠른 실행 속도
- ✅ 작은 바이너리 크기
- ❌ 디버그 어려움

### 권장 워크플로우

1. **개발 중**: `./scripts/quick-dev.sh` 사용
2. **테스트 완료 후**: Release 빌드로 최종 확인
3. **배포 전**: Release 빌드 필수

## 🔧 추가 최적화

### 1. cargo-watch 사용 (파일 변경 시 자동 빌드)

```bash
# 설치
cargo install cargo-watch

# 자동 재빌드
cargo watch -x 'build' -s './scripts/quick-dev.sh'
```

### 2. sccache 사용 (컴파일 캐시)

```bash
# 설치
cargo install sccache

# 환경변수 설정 (.bashrc 또는 .zshrc에 추가)
export RUSTC_WRAPPER=sccache

# 캐시 통계
sccache --show-stats
```

### 3. Incremental 컴파일 확인

`.cargo/config.toml`에 이미 설정되어 있음:
```toml
[build]
incremental = true
```

## 🌐 브라우저 접속

서버 시작 후:
- VM 외부: http://34.50.24.115
- VM 내부: http://localhost:3001

## 🐛 문제 해결

### 포트가 이미 사용 중

```bash
# 이전 서버 프로세스 확인
ps aux | grep server

# 모두 종료
pkill -f "target/debug/server"
pkill -f "target/release/server"
```

### 빌드 에러

```bash
# 의존성 업데이트
cargo update

# 클린 빌드
cargo clean
cargo build
```

### 데이터베이스 문제

```bash
# 데이터베이스 재생성 (주의: 데이터 삭제됨)
rm /home/cosmos/anyon/data/anyon.db
export DATABASE_URL="sqlite:///home/cosmos/anyon/data/anyon.db"
sqlx database create
sqlx migrate run --source crates/db/migrations
```

## 📚 추가 명령어

```bash
# 타입 체크만
cargo check

# 린트
cargo clippy

# 테스트
cargo test

# 특정 크레이트만 빌드
cargo build -p server
```
