# Git Worktree 사용 가이드

## 기본 명령어

### 1. Worktree 생성

```bash
# 새 브랜치를 만들면서 worktree 생성
git worktree add <경로> -b <브랜치명>

# 기존 브랜치를 체크아웃하면서 worktree 생성
git worktree add <경로> <브랜치명>

# 예시
git worktree add ../any-on-user1 -b user1/feature-a
git worktree add ../any-on-user2 user2/feature-b
```

### 2. Worktree 목록 확인

```bash
# 모든 worktree 목록 보기
git worktree list

# 상세 정보 포함
git worktree list --verbose
```

### 3. Worktree로 이동

```bash
# 일반적인 디렉토리 이동과 동일
cd ../any-on-user1
```

### 4. Worktree 삭제

```bash
# worktree 삭제 (브랜치는 유지)
git worktree remove <경로>

# 또는 간단하게
git worktree remove ../any-on-user1

# 강제 삭제 (작업 디렉토리에 변경사항이 있어도)
git worktree remove --force <경로>
```

### 5. Worktree 정리

```bash
# 삭제된 worktree의 참조 정리
git worktree prune

# 자동 정리 (Git 2.17+)
git worktree prune --verbose
```

## 실전 사용 예시

### 3명이 함께 작업하는 경우

```bash
# 사용자1: 새 브랜치로 worktree 생성
git worktree add ../any-on-user1 -b user1/feature-a

# 사용자2: 기존 브랜치로 worktree 생성
git worktree add ../any-on-user2 feat/cloud-terminal-try2

# 사용자3: main 브랜치로 worktree 생성
git worktree add ../any-on-user3 main

# 각자 자신의 worktree로 이동해서 작업
cd ../any-on-user1
# ... 작업 ...

cd ../any-on-user2
# ... 작업 ...

cd ../any-on-user3
# ... 작업 ...
```

### Worktree에서 일반 Git 명령어 사용

```bash
# worktree에서도 일반 git 명령어 그대로 사용 가능
cd ../any-on-user1

git status
git add .
git commit -m "작업 내용"
git push origin user1/feature-a
git pull origin main
```

### Worktree 간 변경사항 확인

```bash
# 원본에서 다른 worktree의 브랜치 확인
git log user1/feature-a

# 다른 worktree의 변경사항 가져오기
git fetch origin user1/feature-a
```

## 유용한 팁

### 현재 worktree 확인

```bash
# 현재 worktree 경로 확인
git rev-parse --git-dir
# 또는
git worktree list | grep "$(pwd)"
```

### Worktree에서 원격 브랜치 가져오기

```bash
cd ../any-on-user1
git fetch origin
git checkout -b user1/new-feature origin/user1/new-feature
```

### Worktree 제한 확인

```bash
# Git은 기본적으로 최대 5개의 worktree를 허용
# 제한 확인
git config --get extensions.worktreeConfig

# 제한 변경 (필요시)
git config extensions.worktreeConfig true
```

## 주의사항

1. **같은 브랜치는 여러 worktree에서 체크아웃 불가**
   ```bash
   # ❌ 불가능: 같은 브랜치를 두 worktree에서 사용
   git worktree add ../worktree1 main
   git worktree add ../worktree2 main  # 에러!
   ```

2. **Worktree 삭제 전에 변경사항 커밋/저장**
   ```bash
   # 변경사항이 있으면 먼저 처리
   git add .
   git commit -m "작업 완료"
   # 그 다음 삭제
   git worktree remove ../any-on-user1
   ```

3. **각 worktree는 독립된 디렉토리**
   - 파일 수정이 서로 영향을 주지 않음
   - 하지만 같은 `.git` 저장소를 공유하므로 Git 인덱스 충돌은 방지됨

## 실전 워크플로우 예시

```bash
# 1. 메인 저장소에서 시작
cd /home/cosmos/any-on

# 2. 각 사용자별 worktree 생성
git worktree add ../any-on-user1 -b user1/feature-a
git worktree add ../any-on-user2 -b user2/feature-b
git worktree add ../any-on-user3 -b user3/feature-c

# 3. 각자 자신의 worktree로 이동
# 사용자1
cd ../any-on-user1
# 작업 후
git add .
git commit -m "user1 작업"
git push origin user1/feature-a

# 사용자2
cd ../any-on-user2
# 작업 후
git add .
git commit -m "user2 작업"
git push origin user2/feature-b

# 4. 작업 완료 후 worktree 삭제
cd /home/cosmos/any-on
git worktree remove ../any-on-user1
git worktree remove ../any-on-user2
git worktree remove ../any-on-user3
```



