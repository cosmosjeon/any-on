# GitHub OAuth 설정 가이드 (Slit)

## 개요
Anyon은 GitHub OAuth를 사용하여 사용자 인증을 처리합니다. 자신의 GitHub OAuth App을 설정하여 브랜딩을 커스터마이징할 수 있습니다.

## 1단계: GitHub OAuth App 생성

### 1.1 GitHub 개발자 설정 접속
```
https://github.com/settings/developers
```

### 1.2 새 OAuth App 생성
- 오른쪽 상단의 **"New OAuth App"** 버튼 클릭

### 1.3 앱 정보 입력

#### 개발 환경 설정 (로컬 테스트)
```
Application name: Slit Anyon Dev
Homepage URL: http://localhost:3001
Application description: Slit AI Coding Agent (개발용)
Authorization callback URL: http://localhost:3001/api/auth/callback
```

#### 프로덕션 환경 설정
```
Application name: Slit Anyon
Homepage URL: https://your-production-domain.com
Application description: Slit AI Coding Agent
Authorization callback URL: https://your-production-domain.com/api/auth/callback
```

### 1.4 App 등록
- **"Register application"** 버튼 클릭

## 2단계: Device Flow 활성화

Device Flow는 CLI 애플리케이션에서 사용자 인증을 간편하게 만들어줍니다.

1. OAuth App 설정 페이지로 이동
2. **"Enable Device Flow"** 체크박스를 활성화
3. **"Update application"** 버튼 클릭하여 저장

## 3단계: Client ID 복사

OAuth App 설정 페이지에서:
- **Client ID** 값을 복사 (형식: `Iv1.xxxxxxxxxxxxxxxx`)
- Client Secret은 Device Flow에서 사용하지 않으므로 필요 없음

## 4단계: 환경 변수 설정

### 개발 환경 (.env 파일)

프로젝트 루트의 `.env` 파일을 열고:

```bash
# GitHub OAuth (필수)
GITHUB_CLIENT_ID=Iv1.YOUR_CLIENT_ID_HERE
```

**예시:**
```bash
GITHUB_CLIENT_ID=Iv1.b507a08c87ecfe98
```

### 프로덕션 환경 (빌드 시)

프로덕션 빌드 시 환경 변수로 설정:

```bash
# 빌드 시 설정
export GITHUB_CLIENT_ID=Iv1.YOUR_CLIENT_ID_HERE
pnpm run build
```

또는 빌드 명령과 함께:

```bash
GITHUB_CLIENT_ID=Iv1.YOUR_CLIENT_ID_HERE pnpm run build
```

## 5단계: 개발 서버 재시작

환경 변수를 변경한 후 개발 서버를 재시작하세요:

```bash
# 개발 서버 종료 (Ctrl+C)
# 다시 시작
pnpm run dev
```

## 6단계: 테스트

### 로컬에서 테스트
1. 브라우저에서 `http://localhost:3001` 접속
2. GitHub 로그인 버튼 클릭
3. GitHub 인증 페이지로 리디렉션되는지 확인
4. 인증 완료 후 앱으로 돌아오는지 확인

### Device Flow 테스트 (CLI)
1. `anyon` CLI 실행
2. GitHub 인증 코드가 표시되는지 확인
3. https://github.com/login/device 에서 코드 입력
4. 인증 성공 확인

## 추가 설정 (선택사항)

### 여러 환경에서 사용
개발/스테이징/프로덕션 환경별로 각각 다른 OAuth App을 생성하는 것을 권장합니다:

```bash
# 개발 환경
GITHUB_CLIENT_ID=Iv1.dev_client_id

# 스테이징 환경
GITHUB_CLIENT_ID=Iv1.staging_client_id

# 프로덕션 환경
GITHUB_CLIENT_ID=Iv1.prod_client_id
```

### Scopes 설정
Anyon은 다음 GitHub 권한이 필요합니다:
- `user:email` - 사용자 이메일 읽기
- `repo` - 저장소 접근 (코드 읽기/쓰기)

이는 OAuth App 설정에서 자동으로 요청됩니다.

## 문제 해결

### "GITHUB_CLIENT_ID environment variable must be set" 에러
- `.env` 파일에 `GITHUB_CLIENT_ID`가 설정되어 있는지 확인
- 개발 서버를 재시작했는지 확인

### 인증이 실패하는 경우
- Client ID가 올바른지 확인
- Device Flow가 활성화되어 있는지 확인
- Callback URL이 정확한지 확인

### 프로덕션에서 작동하지 않는 경우
- 빌드 시 `GITHUB_CLIENT_ID`를 설정했는지 확인
- Callback URL이 프로덕션 도메인과 일치하는지 확인

## 참고 링크

- [GitHub OAuth Apps 문서](https://docs.github.com/en/developers/apps/building-oauth-apps)
- [Device Flow 가이드](https://docs.github.com/en/developers/apps/building-oauth-apps/authorizing-oauth-apps#device-flow)
