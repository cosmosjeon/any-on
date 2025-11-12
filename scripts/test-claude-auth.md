# Claude Code 로그인/로그아웃 테스트 가이드

## 목적
설정 화면에서 Claude Code 로그인/로그아웃 기능이 제대로 작동하는지 검증합니다.

## 테스트 방법

### 1. 현재 로그인 상태 확인

브라우저 개발자 도구 콘솔에서 실행:

```javascript
// 현재 시스템 정보 확인
const response = await fetch('/api/info');
const info = await response.json();
console.log('Claude 연결 상태:', info.claude_secret_state);
console.log('has_credentials:', info.claude_secret_state?.has_credentials);
```

### 2. 로그아웃 테스트

#### 방법 A: 설정 화면에서 직접 테스트
1. 설정 화면 (`/settings/general`)으로 이동
2. "Claude Code Login" 섹션 확인
3. 현재 상태가 "연결 완료"인지 확인
4. "Disconnect" 버튼 클릭
5. 상태가 "연결되지 않음"으로 변경되는지 확인

#### 방법 B: API 직접 호출
브라우저 개발자 도구 콘솔에서 실행:

```javascript
// 로그아웃 API 호출
const logoutResponse = await fetch('/api/auth/claude/logout', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
});
const logoutResult = await logoutResponse.json();
console.log('로그아웃 결과:', logoutResult);

// 상태 재확인
const checkResponse = await fetch('/api/info');
const checkInfo = await checkResponse.json();
console.log('로그아웃 후 상태:', checkInfo.claude_secret_state);
```

### 3. 로그인 테스트

#### 방법 A: 설정 화면에서 직접 테스트
1. 설정 화면 (`/settings/general`)으로 이동
2. "Claude Code Login" 섹션에서 "Connect Claude Account" 버튼 클릭
3. 로그인 다이얼로그가 열리는지 확인
4. CLI 출력이 표시되는지 확인
5. "브라우저로 로그인" 옵션이 나타나면 선택
6. 새 탭에서 Claude 승인 페이지가 열리는지 확인
7. 승인 완료 후 다이얼로그에 "연결 완료" 상태가 표시되는지 확인
8. 다이얼로그 닫기
9. 설정 화면에서 상태가 "연결 완료"로 변경되는지 확인

#### 방법 B: API 직접 호출 (세션 시작)
브라우저 개발자 도구 콘솔에서 실행:

```javascript
// 세션 시작
const startResponse = await fetch('/api/auth/claude/session', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
});
const startResult = await startResponse.json();
console.log('세션 시작:', startResult);

// SSE 스트림 연결 (EventSource 사용)
const sessionId = startResult.data.session_id;
const eventSource = new EventSource(`/api/auth/claude/session/${sessionId}/stream`);

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('CLI 출력:', data);
  
  if (data.type === 'OUTPUT') {
    console.log('출력:', data.line);
  } else if (data.type === 'COMPLETED') {
    console.log('완료:', data.success);
    eventSource.close();
  } else if (data.type === 'ERROR') {
    console.error('오류:', data.message);
    eventSource.close();
  }
};

eventSource.onerror = (error) => {
  console.error('SSE 오류:', error);
  eventSource.close();
};
```

### 4. 전체 테스트 시나리오

다음 순서로 테스트하면 로그인/로그아웃이 제대로 작동하는지 확인할 수 있습니다:

```javascript
// 1. 초기 상태 확인
async function testClaudeAuth() {
  console.log('=== Claude Code 인증 테스트 시작 ===\n');
  
  // 현재 상태 확인
  const initialResponse = await fetch('/api/info');
  const initialInfo = await initialResponse.json();
  const initialConnected = initialInfo.claude_secret_state?.has_credentials;
  console.log('1. 초기 연결 상태:', initialConnected ? '연결됨' : '연결 안 됨');
  
  // 2. 로그아웃 (연결되어 있는 경우)
  if (initialConnected) {
    console.log('\n2. 로그아웃 시도...');
    const logoutResponse = await fetch('/api/auth/claude/logout', {
      method: 'POST',
    });
    const logoutResult = await logoutResponse.json();
    console.log('   로그아웃 결과:', logoutResult.success ? '성공' : '실패');
    
    // 상태 재확인
    const afterLogoutResponse = await fetch('/api/info');
    const afterLogoutInfo = await afterLogoutResponse.json();
    const afterLogoutConnected = afterLogoutInfo.claude_secret_state?.has_credentials;
    console.log('   로그아웃 후 상태:', afterLogoutConnected ? '연결됨 (오류!)' : '연결 안 됨 (정상)');
    
    if (afterLogoutConnected) {
      console.error('   ❌ 로그아웃 실패: 여전히 연결 상태입니다!');
      return;
    }
    console.log('   ✅ 로그아웃 성공');
  }
  
  // 3. 로그인 테스트는 수동으로 진행
  console.log('\n3. 로그인 테스트:');
  console.log('   설정 화면에서 "Connect Claude Account" 버튼을 클릭하여 수동으로 테스트하세요.');
  console.log('   로그인 완료 후 이 스크립트를 다시 실행하여 상태를 확인하세요.');
  
  // 4. 최종 상태 확인
  console.log('\n4. 최종 상태 확인:');
  const finalResponse = await fetch('/api/info');
  const finalInfo = await finalResponse.json();
  const finalConnected = finalInfo.claude_secret_state?.has_credentials;
  console.log('   최종 연결 상태:', finalConnected ? '연결됨' : '연결 안 됨');
  
  console.log('\n=== 테스트 완료 ===');
}

// 실행
testClaudeAuth();
```

## 예상 결과

### 정상 작동 시:
1. **로그아웃 전**: `claude_secret_state.has_credentials = true`
2. **로그아웃 후**: `claude_secret_state.has_credentials = false`
3. **로그인 후**: `claude_secret_state.has_credentials = true`
4. 설정 화면의 상태 표시도 위 값과 일치해야 함

### 문제 발생 시 확인 사항:
1. 브라우저 콘솔에 에러 메시지가 있는지 확인
2. 네트워크 탭에서 API 호출이 성공했는지 확인
3. `reloadSystem()`이 제대로 호출되었는지 확인
4. SecretStore에서 실제로 삭제되었는지 확인 (백엔드 로그 확인)

## 백엔드 로그 확인

서버 로그에서 다음을 확인할 수 있습니다:

```bash
# 로그아웃 시
grep "claude.*logout" logs.txt

# SecretStore 삭제 확인
grep "SECRET_CLAUDE_ACCESS" logs.txt
```

## 참고 파일

- 프론트엔드: `frontend/src/pages/settings/GeneralSettings.tsx`
- 로그인 다이얼로그: `frontend/src/components/dialogs/auth/ClaudeLoginDialog.tsx`
- API 엔드포인트: `crates/server/src/routes/auth.rs`
- 로그아웃 핸들러: `crates/server/src/routes/auth.rs:389-397`


