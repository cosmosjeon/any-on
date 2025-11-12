# Worktree 작업 합치기 가이드

## 방법 1: Pull Request 사용 (권장) ⭐

### 1단계: 각 worktree에서 작업하고 커밋

```bash
# user1의 worktree에서
cd /home/cosmos/any-on-user1

# 파일 수정 후
git add .
git commit -m "user1의 작업 내용"
git push origin user1/work
```

### 2단계: GitHub/GitLab에서 Pull Request 생성

- GitHub/GitLab 웹사이트 접속
- `user1/work` 브랜치 → `main` (또는 `feat/cloud-terminal-try2`)로 PR 생성
- 코드 리뷰 후 Merge

### 3단계: 원본 브랜치에서 최신 내용 가져오기

```bash
# 원본 worktree로 이동
cd /home/cosmos/any-on

# 원격의 최신 내용 가져오기
git fetch origin

# main 브랜치로 전환하고 merge된 내용 가져오기
git checkout main
git pull origin main

# 또는 feat/cloud-terminal-try2 브랜치에서
git checkout feat/cloud-terminal-try2
git pull origin feat/cloud-terminal-try2
```

---

## 방법 2: 로컬에서 직접 Merge

### 1단계: 각 worktree에서 작업하고 커밋

```bash
cd /home/cosmos/any-on-user1
git add .
git commit -m "user1 작업"
git push origin user1/work
```

### 2단계: 원본 브랜치에서 merge

```bash
# 원본 worktree로 이동
cd /home/cosmos/any-on

# 합치고 싶은 브랜치로 전환 (예: main)
git checkout main

# user1의 브랜치를 merge
git merge user1/work

# 충돌이 있으면 해결 후
git add .
git commit -m "Merge user1/work into main"

# 원격에 push
git push origin main
```

---

## 방법 3: Rebase 사용 (히스토리 깔끔하게)

```bash
# 원본 브랜치로 이동
cd /home/cosmos/any-on
git checkout main
git pull origin main

# user1의 브랜치를 rebase
git rebase user1/work

# 또는 user1 브랜치를 main 위에 rebase
git checkout user1/work
git rebase main
git push origin user1/work --force-with-lease
```

---

## 실제 예시: 3명이 작업하는 경우

### 각자 작업
```bash
# 사용자1
cd /home/cosmos/any-on-user1
# 작업 후
git add .
git commit -m "user1: 기능 A 구현"
git push origin user1/work

# 사용자2
cd /home/cosmos/any-on-user2
# 작업 후
git add .
git commit -m "user2: 기능 B 구현"
git push origin user2/work

# 사용자3
cd /home/cosmos/any-on-user3
# 작업 후
git add .
git commit -m "user3: 기능 C 구현"
git push origin user3/work
```

### 합치기 (PR 방식)
1. GitHub에서 각각 PR 생성
2. 리뷰 후 순차적으로 Merge
3. 원본에서 pull

```bash
cd /home/cosmos/any-on
git checkout main
git pull origin main  # 모든 변경사항 가져오기
```

---

## 주의사항

1. **충돌 해결**: 같은 파일을 수정했으면 충돌 발생 가능
2. **최신 상태 유지**: merge 전에 원본 브랜치를 최신으로 업데이트
   ```bash
   git checkout main
   git pull origin main
   git merge user1/work
   ```
3. **Worktree 업데이트**: merge 후 각 worktree도 업데이트
   ```bash
   cd /home/cosmos/any-on-user1
   git fetch origin
   git rebase origin/main  # 또는 merge
   ```



