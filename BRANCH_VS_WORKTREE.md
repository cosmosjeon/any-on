# 브랜치 vs Worktree 차이점

## 문제 상황: 3명이 같은 VM에서 작업

### ❌ 일반 브랜치 사용 (같은 폴더)

```
사용자1: cd /home/cosmos/any-on && git checkout user1/work
사용자2: cd /home/cosmos/any-on && git checkout user2/work  
사용자3: cd /home/cosmos/any-on && git checkout user3/work
```

**문제점:**
1. **같은 폴더 사용** → 파일 수정 시 충돌
2. **node_modules 충돌** → 한 명이 npm install 하면 다른 사람 영향
3. **빌드 결과물 충돌** → target/, dist/ 등이 덮어써짐
4. **Git 인덱스 충돌** → .git/index 파일 동시 접근 시 에러
5. **환경 변수 충돌** → .env 파일이 공유됨
6. **포트 충돌** → 개발 서버 포트가 겹침

**예시:**
```bash
# 사용자1이 파일 수정
echo "user1 작업" > test.txt
git add test.txt

# 사용자2가 같은 파일 수정 (같은 폴더!)
echo "user2 작업" > test.txt  # user1의 변경사항 덮어씀!
```

---

### ✅ Worktree 사용 (다른 폴더)

```
사용자1: cd /home/cosmos/any-on-user1 (독립된 폴더)
사용자2: cd /home/cosmos/any-on-user2 (독립된 폴더)
사용자3: cd /home/cosmos/any-on-user3 (독립된 폴더)
```

**장점:**
1. **독립된 폴더** → 파일 수정 시 충돌 없음
2. **독립된 node_modules** → 각자 npm install 가능
3. **독립된 빌드 결과물** → 서로 영향 없음
4. **독립된 Git 인덱스** → 동시 커밋 가능
5. **독립된 환경 변수** → 각자 .env 파일 사용 가능
6. **독립된 포트** → 각자 다른 포트 사용 가능

**예시:**
```bash
# 사용자1이 파일 수정
cd /home/cosmos/any-on-user1
echo "user1 작업" > test.txt  # 독립된 폴더!

# 사용자2가 같은 파일 수정 (다른 폴더!)
cd /home/cosmos/any-on-user2
echo "user2 작업" > test.txt  # 충돌 없음! 다른 폴더!
```

---

## 핵심 차이점 요약

| 항목 | 일반 브랜치 | Worktree |
|------|------------|----------|
| **폴더** | 같은 폴더 | 다른 폴더 |
| **파일 충돌** | 발생 가능 | 없음 |
| **빌드 아티팩트** | 공유됨 | 독립적 |
| **동시 작업** | 어려움 | 쉬움 |
| **Git 인덱스** | 충돌 가능 | 독립적 |
| **환경 설정** | 공유됨 | 독립적 |

---

## 실제 사용 예시

### 일반 브랜치 (문제 있음)
```bash
# 사용자1
cd /home/cosmos/any-on
git checkout -b user1/work
npm install  # 다른 사람에게 영향!
pnpm run dev  # 포트 충돌 가능!

# 사용자2 (같은 폴더!)
cd /home/cosmos/any-on
git checkout -b user2/work
npm install  # user1의 node_modules 덮어씀!
pnpm run dev  # 포트 충돌!
```

### Worktree (문제 없음)
```bash
# 사용자1
cd /home/cosmos/any-on-user1
npm install  # 독립된 node_modules
pnpm run dev  # 포트 3000 사용

# 사용자2 (다른 폴더!)
cd /home/cosmos/any-on-user2
npm install  # 독립된 node_modules, 충돌 없음!
pnpm run dev  # 포트 3001 사용 (다른 포트)
```

---

## 결론

**일반 브랜치**: 같은 폴더에서 브랜치만 바꿈 → 충돌 발생
**Worktree**: 다른 폴더에서 작업 → 충돌 없음

**같은 VM에서 여러 명이 작업할 때는 Worktree가 필수입니다!**



