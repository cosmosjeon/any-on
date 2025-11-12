# 클라우드 백엔드 + 로컬 프론트엔드 개발 가이드

이 가이드는 백엔드를 클라우드 VM에 배포하고, 로컬에서 프론트엔드 개발 서버를 실행하여 테스트하는 방법을 설명합니다.

## 🎯 개요

```
┌──────────────────┐         HTTP/WebSocket         ┌──────────────────┐
│   로컬 PC        │  ───────────────────────────>  │   클라우드 VM     │
│                  │                                 │                  │
│  Frontend Dev    │  <───────────────────────────  │  Backend Server  │
│  (Port 3000)     │                                 │  (Port 3000)     │
└──────────────────┘                                 └──────────────────┘
```

## 📋 사전 준비

### 클라우드 VM 요구사항
- Ubuntu 20.04+ 또는 다른 Linux 배포판
- Rust 1.70+ 설치
- Git 설치
- 포트 3000 (또는 원하는 포트) 개방

### 로컬 개발 환경
- Node.js 18+
- pnpm 설치
- Git

---

## 🚀 1단계: 클라우드 VM에 백엔드 배포

### 1.1 저장소 클론

```bash
# SSH로 VM 접속
ssh user@your-vm-ip

# 저장소 클론
cd ~
git clone https://github.com/your-org/any-on.git
cd any-on
```

### 1.2 .env.cloud 파일 설정

`.env.cloud` 파일을 생성하거나 수정합니다:

```bash
# .env.cloud 파일 편집
nano .env.cloud
```

필수 설정:

```bash
# 암호화 키 (새로 생성 권장)
ANYON_SECRET_KEY=your_secret_key_here

# 서버 설정 (중요!)
BACKEND_PORT=3000
HOST=0.0.0.0  # 외부 접근 허용

# 디렉토리 설정
ANYON_CLOUD_BASE_DIR=/home/user/anyon
ANYON_ASSET_DIR=/home/user/anyon/data
ANYON_TEMP_DIR=/home/user/anyon/tmp
ANYON_WORKTREE_DIR=/home/user/anyon/worktrees
ANYON_WORKSPACE_DIR=/home/user/anyon/workspace
ANYON_DATABASE_FILE=/home/user/anyon/data/anyon.db

# CORS 설정 (로컬 개발용)
# 주석 해제하고 로컬 IP를 추가하세요
# CORS_ALLOWED_ORIGINS=http://localhost:3000

# GitHub OAuth
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_client_secret
```

**중요:** `HOST=0.0.0.0`으로 설정해야 외부에서 접근할 수 있습니다!

### 1.3 백엔드 빌드 및 실행

```bash
# cloud feature를 활성화하여 빌드
cargo build --release --features cloud

# 실행
./target/release/server
```

또는 직접 실행:

```bash
cargo run --release --features cloud
```

**서버가 시작되면 다음과 같은 로그를 확인하세요:**
```
Server running on http://0.0.0.0:3000
```

### 1.4 방화벽 설정 (필요시)

```bash
# UFW 사용 시
sudo ufw allow 3000/tcp

# 또는 iptables 사용 시
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
```

### 1.5 백엔드 접근 테스트

로컬 PC에서 테스트:

```bash
# health check 엔드포인트 테스트
curl http://your-vm-ip:3000/health
```

정상 응답:
```json
{"status": "ok"}
```

---

## 💻 2단계: 로컬에서 프론트엔드 개발 서버 실행

### 2.1 로컬 환경변수 설정

로컬 PC의 `.env` 파일을 수정합니다:

```bash
# .env 파일 편집
nano .env
```

다음 설정을 변경:

```bash
# 프론트엔드 포트
FRONTEND_PORT=3000

# 백엔드 호스트 - VM의 IP 또는 도메인으로 변경!
BACKEND_HOST=123.45.67.89  # 또는 api.yourapp.com

# 백엔드 포트
BACKEND_PORT=3000

# 브라우저 자동 열기 (선택사항)
VITE_OPEN=false
```

**예시:**
- VM IP가 `192.168.1.100`이면: `BACKEND_HOST=192.168.1.100`
- 도메인이 있으면: `BACKEND_HOST=api.yourapp.com`

### 2.2 프론트엔드 실행

```bash
# 의존성 설치 (최초 1회)
pnpm install

# 프론트엔드 개발 서버만 실행
pnpm run frontend:dev
```

### 2.3 브라우저에서 확인

브라우저에서 `http://localhost:3000`을 엽니다.

---

## 🔍 3단계: 동작 확인

### 3.1 네트워크 요청 확인

브라우저 개발자 도구 (F12) → Network 탭에서 다음을 확인:

- API 요청이 `http://your-vm-ip:3000/api/...`로 전송되는지
- 응답이 정상적으로 오는지
- CORS 에러가 없는지

### 3.2 CORS 에러 발생 시

만약 CORS 에러가 발생한다면:

1. VM의 `.env.cloud` 파일에 다음 추가:
   ```bash
   CORS_ALLOWED_ORIGINS=http://localhost:3000
   ```

2. 백엔드 재시작:
   ```bash
   # Ctrl+C로 중지 후 재시작
   ./target/release/server
   ```

---

## 🛠️ 트러블슈팅

### 문제 1: "connection refused" 에러

**원인:** VM에서 백엔드가 실행되지 않았거나 방화벽이 막혀있음

**해결:**
1. VM에서 백엔드가 실행 중인지 확인: `ps aux | grep server`
2. 포트가 열려있는지 확인: `sudo netstat -tlnp | grep 3000`
3. 방화벽 설정 확인

### 문제 2: CORS 에러

**에러 메시지:**
```
Access to fetch at 'http://vm-ip:3000/api/...' from origin 'http://localhost:3000'
has been blocked by CORS policy
```

**해결:**
VM의 `.env.cloud`에 다음 추가 후 재시작:
```bash
CORS_ALLOWED_ORIGINS=http://localhost:3000
```

### 문제 3: "HOST must be 0.0.0.0"

**원인:** VM의 백엔드가 `127.0.0.1`에 바인딩되어 외부 접근 불가

**해결:**
`.env.cloud`에서 `HOST=0.0.0.0`으로 설정했는지 확인

### 문제 4: 환경변수가 적용되지 않음

**해결:**
1. `.env.cloud` 파일이 프로젝트 루트에 있는지 확인
2. 백엔드를 재시작
3. 환경변수 로딩 로그 확인

---

## 📝 추가 팁

### 백그라운드 실행

백엔드를 백그라운드에서 실행하려면:

```bash
# nohup 사용
nohup ./target/release/server > server.log 2>&1 &

# 또는 systemd 서비스 생성
sudo systemctl start anyon-backend
```

### 로그 확인

```bash
# 실시간 로그 보기
tail -f server.log

# 또는 journalctl 사용
sudo journalctl -u anyon-backend -f
```

### 개발 워크플로우

1. 로컬에서 프론트엔드 코드 수정
2. 브라우저에서 hot reload로 즉시 확인
3. 백엔드 API 호출은 클라우드 VM으로 전송
4. 필요시 백엔드 코드도 수정하여 VM에 배포

---

## 🔒 보안 권장사항

1. **프로덕션 환경에서는 HTTPS 사용**
   - Nginx 리버스 프록시 + Let's Encrypt 설정

2. **CORS 설정 제한**
   ```bash
   # 특정 도메인만 허용
   CORS_ALLOWED_ORIGINS=https://yourapp.com,https://www.yourapp.com
   ```

3. **방화벽 설정**
   - 필요한 포트만 개방
   - SSH 포트 변경 권장

4. **환경변수 보호**
   - `.env.cloud` 파일 권한 설정: `chmod 600 .env.cloud`
   - GitHub에 절대 커밋하지 않기

---

## 📚 관련 문서

- [CLAUDE.md](./CLAUDE.md) - 개발 룰 및 워크플로우
- [.env.example](./.env.example) - 환경변수 예시
- [Backend Architecture](./문서/아키텍쳐/백엔드_아키텍쳐.md)

---

## 🆘 도움이 필요하신가요?

- GitHub Issues: [프로젝트 이슈 페이지]
- 문의: [이메일 또는 연락처]
