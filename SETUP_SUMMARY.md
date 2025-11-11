# 설정 완료 요약

## ✅ 현재 상태

### 클라우드 백엔드 (34.50.24.115:3000)
- ✅ **백엔드 실행 중**: http://34.50.24.115:3000
- ✅ **API 응답 정상**: `/api/health`, `/api/projects` 모두 동작
- ✅ **프로젝트 3개 발견**:
  - 테스트
  - 전도현
  - any-on

### 로컬 설정
- ✅ **`.env` 업데이트 완료**: `BACKEND_HOST=34.50.24.115`, `BACKEND_PORT=3000`
- ✅ **CORS 코드 추가 완료**: `crates/server/src/routes/mod.rs`

---

## ⚠️ 중요: 클라우드 백엔드 재시작 필요

현재 클라우드에서 실행 중인 백엔드는 **CORS 설정이 없는 이전 버전**입니다.

### 왜 재시작이 필요한가?
- 새로 추가한 CORS 코드가 아직 적용되지 않았음
- CORS 헤더가 없으면 브라우저가 API 요청을 차단함

### 재시작 방법 (VM에서 실행)

#### 옵션 1: 기존 프로세스 재시작 (가장 간단)
```bash
# VM에 SSH 접속
ssh user@34.50.24.115

# 프로젝트 디렉토리로 이동
cd ~/any-on  # 또는 프로젝트가 있는 경로

# 최신 코드 pull
git pull

# 기존 서버 중지
pkill -f "target/release/server"

# 새로 빌드 및 실행
cargo build --release --features cloud
nohup ./target/release/server > server.log 2>&1 &

# 확인
tail -f server.log
```

#### 옵션 2: 배포 스크립트 사용 (권장)
```bash
# VM에 SSH 접속
ssh user@34.50.24.115

cd ~/any-on
git pull

# 배포 스크립트 실행 (자동으로 기존 프로세스 중지 후 재시작)
./scripts/deploy-cloud.sh
```

#### 옵션 3: systemd 서비스로 실행 중인 경우
```bash
ssh user@34.50.24.115

cd ~/any-on
git pull
cargo build --release --features cloud

sudo systemctl restart anyon-backend
sudo systemctl status anyon-backend
```

---

## 🚀 로컬 프론트엔드 실행

백엔드 재시작 후:

```bash
# 로컬 PC에서
cd ~/Documents/dev/any-on

# 방법 1: 스크립트 사용
./scripts/start-local-frontend.sh

# 방법 2: 직접 실행
cd frontend
pnpm run dev
```

브라우저에서 `http://localhost:3000` 접속

---

## 🔍 테스트 체크리스트

### 1. 백엔드 재시작 확인
```bash
# 로컬에서 테스트
curl http://34.50.24.115:3000/api/health

# CORS 헤더 확인 (재시작 후)
curl -v -X OPTIONS http://34.50.24.115:3000/api/projects \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET" \
  2>&1 | grep -i "access-control"
```

예상 결과: `access-control-allow-origin: *` 헤더가 보여야 함

### 2. 프론트엔드 연결 확인
1. 브라우저 개발자 도구 열기 (F12)
2. Network 탭 확인
3. API 요청이 `http://34.50.24.115:3000/api/...`로 전송되는지 확인
4. CORS 에러가 없는지 확인

---

## 🐛 문제 해결

### CORS 에러가 계속 발생하는 경우

#### 해결 방법 1: VM의 .env.cloud에 명시적으로 설정
```bash
# VM에서
cd ~/any-on
nano .env.cloud
```

다음 라인 추가:
```bash
CORS_ALLOWED_ORIGINS=http://localhost:3000
```

재시작:
```bash
pkill -f "target/release/server"
./target/release/server
```

#### 해결 방법 2: 코드 직접 확인
```bash
# VM에서
cd ~/any-on
cat crates/server/src/routes/mod.rs | grep -A 20 "Configure CORS"
```

CORS 코드가 있어야 함.

### 연결이 안 되는 경우

```bash
# 포트가 열려있는지 확인
curl http://34.50.24.115:3000/api/health

# 방화벽 확인 (VM에서)
sudo ufw status
sudo ufw allow 3000/tcp
```

---

## 📝 다음 단계

1. ✅ VM에 SSH 접속
2. ✅ 프로젝트 디렉토리로 이동
3. ✅ `git pull` 실행
4. ✅ 백엔드 재시작 (위의 방법 중 하나)
5. ✅ CORS 헤더 확인
6. ✅ 로컬에서 프론트엔드 실행
7. ✅ 브라우저에서 테스트

---

## 📞 도움이 필요하면

- CORS 헤더가 보이지 않는 경우 → 백엔드가 새 코드로 실행되지 않은 것
- "connection refused" 에러 → 백엔드가 실행되지 않은 것
- 다른 포트로 실행 중인지 확인: `ps aux | grep server`
