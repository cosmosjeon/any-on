/**
 * Claude Code 로그인/로그아웃 테스트 스크립트
 * 
 * 사용법:
 * 1. 브라우저 개발자 도구 콘솔을 엽니다 (F12)
 * 2. 이 스크립트를 복사하여 콘솔에 붙여넣고 실행합니다
 * 3. 테스트 결과가 콘솔에 출력됩니다
 */

(async function testClaudeAuth() {
  console.log('=== Claude Code 인증 테스트 시작 ===\n');
  
  try {
    // 1. 초기 상태 확인
    console.log('1️⃣ 초기 연결 상태 확인 중...');
    const initialResponse = await fetch('/api/info');
    if (!initialResponse.ok) {
      throw new Error(`API 호출 실패: ${initialResponse.status}`);
    }
    const initialInfo = await initialResponse.json();
    const initialConnected = initialInfo.claude_secret_state?.has_credentials ?? false;
    console.log(`   현재 상태: ${initialConnected ? '✅ 연결됨' : '❌ 연결 안 됨'}`);
    console.log(`   상세 정보:`, initialInfo.claude_secret_state);
    
    // 2. 로그아웃 테스트 (연결되어 있는 경우)
    if (initialConnected) {
      console.log('\n2️⃣ 로그아웃 테스트 시작...');
      const logoutResponse = await fetch('/api/auth/claude/logout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!logoutResponse.ok) {
        throw new Error(`로그아웃 API 호출 실패: ${logoutResponse.status}`);
      }
      
      const logoutResult = await logoutResponse.json();
      console.log(`   API 응답:`, logoutResult);
      
      // 상태 재확인
      await new Promise(resolve => setTimeout(resolve, 500)); // 잠시 대기
      const afterLogoutResponse = await fetch('/api/info');
      const afterLogoutInfo = await afterLogoutResponse.json();
      const afterLogoutConnected = afterLogoutInfo.claude_secret_state?.has_credentials ?? false;
      
      if (afterLogoutConnected) {
        console.error('   ❌ 로그아웃 실패: 여전히 연결 상태입니다!');
        console.error('   문제: SecretStore에서 삭제되지 않았을 수 있습니다.');
        return;
      }
      console.log(`   ✅ 로그아웃 성공: ${afterLogoutConnected ? '연결됨' : '연결 안 됨'}`);
    } else {
      console.log('\n2️⃣ 로그아웃 테스트 건너뜀 (이미 연결 안 됨)');
    }
    
    // 3. 로그인 안내
    console.log('\n3️⃣ 로그인 테스트:');
    console.log('   📝 수동 테스트 필요:');
    console.log('   1. 설정 화면(/settings/general)으로 이동');
    console.log('   2. "Claude Code Login" 섹션에서 "Connect Claude Account" 버튼 클릭');
    console.log('   3. 로그인 다이얼로그에서 "브라우저로 로그인" 선택');
    console.log('   4. 새 탭에서 Claude 승인 완료');
    console.log('   5. 로그인 완료 후 이 스크립트를 다시 실행하여 상태 확인');
    
    // 4. 최종 상태 확인
    console.log('\n4️⃣ 최종 상태 확인:');
    const finalResponse = await fetch('/api/info');
    const finalInfo = await finalResponse.json();
    const finalConnected = finalInfo.claude_secret_state?.has_credentials ?? false;
    console.log(`   최종 상태: ${finalConnected ? '✅ 연결됨' : '❌ 연결 안 됨'}`);
    console.log(`   상세 정보:`, finalInfo.claude_secret_state);
    
    // 5. 설정 화면 상태와 비교 안내
    console.log('\n5️⃣ 설정 화면 확인:');
    console.log('   설정 화면의 "Claude Code Login" 섹션에서 상태를 확인하세요.');
    console.log(`   예상 표시: "${finalConnected ? 'Claude credentials are stored securely.' : 'Claude credentials have not been connected yet.'}"`);
    
    console.log('\n=== 테스트 완료 ===');
    console.log('\n💡 팁:');
    console.log('   - 로그인 후 이 스크립트를 다시 실행하여 상태 변경을 확인하세요');
    console.log('   - 문제가 있으면 브라우저 콘솔과 네트워크 탭을 확인하세요');
    console.log('   - 백엔드 로그도 확인하여 SecretStore 동작을 검증하세요');
    
  } catch (error) {
    console.error('\n❌ 테스트 중 오류 발생:', error);
    console.error('   스택 트레이스:', error.stack);
  }
})();


