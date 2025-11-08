# Anyon 데이터베이스 아키텍처 문서

## 개요

Anyon 프로젝트의 데이터베이스 아키텍처에 대한 종합 문서입니다. 이 디렉토리는 데이터베이스 스키마, 마이그레이션 이력, 쿼리 패턴, 인덱스 전략 등 데이터베이스와 관련된 모든 정보를 포함합니다.

### 기술 스택

- **데이터베이스**: SQLite 3.35+
- **쿼리 빌더**: SQLx (compile-time checked queries)
- **마이그레이션**: SQLx migrations
- **타입 시스템**: Rust (ts-rs로 TypeScript 자동 생성)

### 주요 특징

1. **이벤트 소싱 패턴**: 모든 실행 이력을 시간순으로 저장
2. **Git 통합**: Commit tracking, branch management, worktree isolation
3. **다형성 데이터 모델**: JSON 기반 executor_action, merge type discrimination
4. **성능 최적화**: 복합 인덱스, partial 인덱스, JSONL 로그
5. **외래 키 CASCADE**: 자동 데이터 정리

---

## 문서 구조

### 📘 [스키마개요.md](./스키마개요.md)
**전체 데이터베이스 구조 한눈에 보기**

- ERD (Mermaid 다이어그램)
- 테이블 목록 및 관계
- 핵심 도메인 모델
- 데이터 무결성 전략
- 설계 철학

**이런 분들께 추천**:
- 프로젝트에 새로 합류한 개발자
- 데이터베이스 구조를 빠르게 파악하고 싶은 분
- 테이블 간 관계를 이해하고 싶은 분

---

### 📗 [테이블상세.md](./테이블상세.md)
**각 테이블의 상세 스펙**

11개 테이블의 완전한 문서:
- 컬럼 정의 (타입, 제약, 설명)
- 인덱스 정보
- 비즈니스 로직
- 관련 쿼리 예시
- 상태 전이도 (FSM)

**이런 분들께 추천**:
- 특정 테이블의 컬럼 의미를 확인하고 싶은 분
- CHECK 제약이나 UNIQUE 조건을 알아야 하는 분
- 비즈니스 로직을 이해하고 싶은 분

---

### 📙 [마이그레이션이력.md](./마이그레이션이력.md)
**데이터베이스 진화 과정**

총 46개 마이그레이션의 전체 타임라인:
- Phase별 주요 변경사항
- Breaking Changes
- 데이터 마이그레이션 전략
- 리팩토링 히스토리

**이런 분들께 추천**:
- 스키마가 왜 이렇게 설계되었는지 궁금한 분
- Breaking Change를 파악하고 싶은 분
- 새로운 마이그레이션을 추가해야 하는 분

---

### 📕 [쿼리패턴.md](./쿼리패턴.md)
**실전 SQL 쿼리 가이드**

코드베이스에서 실제 사용되는 쿼리 패턴:
- CRUD 기본 패턴
- JOIN 패턴 (INNER, LEFT, Self)
- 집계 쿼리 (COUNT, GROUP BY, HAVING)
- 시계열 쿼리
- 복잡한 비즈니스 로직 쿼리
- 성능 최적화 패턴

**이런 분들께 추천**:
- 새로운 쿼리를 작성해야 하는 분
- N+1 문제를 해결하고 싶은 분
- SQLx 타입 매핑을 이해하고 싶은 분

---

### 📓 [인덱스전략.md](./인덱스전략.md)
**쿼리 성능 최적화의 핵심**

모든 인덱스의 설계 의도와 활용법:
- 인덱스 종류 (PRIMARY, UNIQUE, Composite, Partial)
- 현재 적용된 인덱스 전체 목록
- EXPLAIN QUERY PLAN 분석
- 인덱스 최적화 가이드
- 성능 벤치마크

**이런 분들께 추천**:
- 쿼리 성능을 개선하고 싶은 분
- 새로운 인덱스를 추가할지 고민하는 분
- 중복 인덱스를 제거하고 싶은 분

---

## 빠른 시작 가이드

### 1. 데이터베이스 구조 파악
```bash
# 1단계: ERD 확인
cat 스키마개요.md

# 2단계: 주요 테이블 확인
cat 테이블상세.md | grep "##"
```

### 2. 새로운 쿼리 작성
```bash
# 1. 관련 쿼리 패턴 찾기
grep -A 20 "특정 패턴" 쿼리패턴.md

# 2. 테이블 스펙 확인
cat 테이블상세.md | grep -A 50 "## table_name"

# 3. SQLx 쿼리 작성
# crates/db/src/models/에 추가
```

### 3. 마이그레이션 추가
```bash
# 1. 마이그레이션 생성
sqlx migrate add description_here

# 2. SQL 작성 (마이그레이션이력.md 참고)
# crates/db/migrations/YYYYMMDDHHMMSS_description.sql

# 3. 적용
sqlx migrate run

# 4. 타입 생성
npm run generate-types
```

### 4. 성능 문제 해결
```bash
# 1. 슬로우 쿼리 확인
EXPLAIN QUERY PLAN <your_query>

# 2. 인덱스 확인
cat 인덱스전략.md | grep "table_name"

# 3. 인덱스 추가 (마이그레이션으로)
sqlx migrate add add_index_to_table
```

---

## 주요 데이터 흐름

### 1. TaskAttempt 라이프사이클

```
사용자 요청
  ↓
TaskAttempt 생성 (branch 할당)
  ↓
ExecutionProcess (setup_script) 실행
  ↓ (before_head_commit 캡처)
ExecutionProcess (coding_agent) 실행
  ↓ (로그 스트리밍)
ExecutorSession 생성
  ↓ (AI 응답 수신)
ExecutionProcess 완료 (after_head_commit 캡처)
  ↓
Merge 생성 (Direct or PR)
  ↓
Worktree 정리 (72시간 후)
```

### 2. Draft 시스템 (낙관적 UI)

```
사용자 입력
  ↓
Draft 생성/업데이트 (version 증가)
  ↓
queued = TRUE
  ↓
전송 시도 (try_mark_sending)
  ↓ (sending = TRUE, 락 획득)
ExecutionProcess 생성
  ↓
clear_after_send
  - Follow-up: 빈 문자열로 초기화
  - Retry: 레코드 삭제
```

### 3. 로그 스트리밍

```
프로세스 시작
  ↓
ExecutionProcessLogs 생성 (빈 문자열)
  ↓
로그 발생 (stdout/stderr)
  ↓
append_log_line (JSONL 추가)
  ↓ (Server-Sent Events)
프론트엔드 실시간 표시
  ↓
프로세스 종료
  ↓
로그 영구 저장
```

---

## 데이터베이스 설계 원칙

### 1. 정규화 vs 비정규화

**정규화 (3NF)**:
- projects, tasks, task_attempts (핵심 도메인)
- images, task_images (M:N 관계)

**비정규화**:
- ExecutionProcessLogs (JSONL 형식)
- executor_action (JSON 필드)
- image_ids in drafts (JSON 배열)

**트레이드오프**:
- 정규화: 데이터 무결성 ✓, 쿼리 복잡도 ✗
- 비정규화: 쿼리 성능 ✓, 유연성 ✓, 무결성 관리 ✗

---

### 2. 외래 키 전략

**모든 외래 키에 CASCADE 적용**:
```sql
FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
```

**장점**:
- 자동 데이터 정리 (고아 레코드 방지)
- 애플리케이션 로직 간소화

**주의사항**:
- 의도하지 않은 대량 삭제 가능
- 삭제 전 확인 필수

---

### 3. Soft Delete vs Hard Delete

**Hard Delete** (기본):
- projects, tasks 삭제 시 CASCADE

**Soft Delete** (dropped 플래그):
- execution_processes (히스토리 보존)
- Restore 기능 구현 가능

---

### 4. 타임스탬프 전략

**모든 테이블**:
```sql
created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
```

**서브초 정밀도** (`subsec`):
- 마이크로초 단위 저장
- 이벤트 순서 보장

---

## 성능 고려사항

### 데이터베이스 크기 예측

**테이블별 예상 크기** (1,000 프로젝트, 프로젝트당 100 태스크 기준):

| 테이블 | 레코드 수 | 평균 크기 | 총 크기 |
|--------|----------|----------|---------|
| projects | 1,000 | 1 KB | 1 MB |
| tasks | 100,000 | 500 B | 50 MB |
| task_attempts | 200,000 | 300 B | 60 MB |
| execution_processes | 1,000,000 | 400 B | 400 MB |
| execution_process_logs | 1,000,000 | 10 KB | 10 GB |
| executor_sessions | 500,000 | 500 B | 250 MB |
| merges | 100,000 | 300 B | 30 MB |
| drafts | 50,000 | 1 KB | 50 MB |
| images | 10,000 | 200 B | 2 MB |
| task_images | 50,000 | 100 B | 5 MB |
| tags | 100 | 500 B | 50 KB |

**총 예상 크기**: ~11 GB (로그가 대부분)

---

### 쿼리 성능 목표

| 쿼리 타입 | 목표 시간 | 전략 |
|----------|----------|------|
| PRIMARY KEY 조회 | < 1ms | 자동 인덱스 |
| 프로젝트별 태스크 목록 | < 10ms | 복합 인덱스 |
| 실행 프로세스 히스토리 | < 50ms | 복합 인덱스 + LIMIT |
| 로그 스트리밍 | < 100ms | JSONL append |
| 전체 통계 (COUNT) | < 500ms | 캐싱 고려 |

---

### 확장성 전략

**현재** (SQLite):
- 단일 파일 데이터베이스
- 동시 쓰기 제한 (WAL 모드로 개선 가능)
- 최대 ~1TB 권장

**미래** (확장 옵션):
- **PostgreSQL 마이그레이션**: 동시성 개선
- **읽기 복제본**: 분석 쿼리 분리
- **샤딩**: 프로젝트별 DB 분리
- **아카이브**: 오래된 로그 별도 저장

---

## 트러블슈팅

### 자주 발생하는 문제

#### 1. FOREIGN KEY constraint failed
```
원인: 외래 키 제약 위반
해결:
- PRAGMA foreign_keys = ON 확인
- 참조 무결성 검증
- CASCADE 동작 이해
```

#### 2. UNIQUE constraint failed
```
원인: 중복 값 삽입 시도
해결:
- UNIQUE 제약 확인 (테이블상세.md)
- UPSERT 패턴 사용 (ON CONFLICT)
```

#### 3. SQLx compile error
```
원인: 쿼리 타입 불일치
해결:
- 타입 주석 명시 (as "col!: Type")
- RETURNING 절 추가
- Enum 이름 확인
```

#### 4. Slow query
```
원인: 인덱스 미활용
해결:
1. EXPLAIN QUERY PLAN 확인
2. 인덱스전략.md 참고
3. 복합 인덱스 추가 고려
```

---

## 개발 워크플로우

### 새로운 기능 추가 시

```bash
# 1. 스키마 변경 필요성 판단
# - 새 테이블 필요? → 마이그레이션
# - 기존 테이블 확장? → 컬럼 추가 마이그레이션

# 2. 마이그레이션 작성
sqlx migrate add add_feature_x

# 3. 모델 업데이트
# crates/db/src/models/에 구조체 추가/수정

# 4. 타입 생성
npm run generate-types

# 5. 쿼리 작성 (쿼리패턴.md 참고)

# 6. 인덱스 고려 (인덱스전략.md 참고)

# 7. 테스트
cargo test -p db

# 8. 문서 업데이트 (이 디렉토리)
```

---

## 참고 자료

### 공식 문서
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [SQLx GitHub](https://github.com/launchbadge/sqlx)
- [ts-rs GitHub](https://github.com/Aleph-Alpha/ts-rs)

### 관련 프로젝트 문서
- [CLAUDE.md](../../CLAUDE.md) - 개발 워크플로우
- [crates/db/README.md](../../crates/db/README.md) - DB 크레이트 문서

### 학습 자료
- [Use The Index, Luke](https://use-the-index-luke.com/) - 인덱스 최적화
- [SQLite Query Planner](https://www.sqlite.org/queryplanner.html) - 쿼리 최적화

---

## 기여 가이드

### 문서 업데이트

이 문서는 스키마 변경 시 항상 함께 업데이트되어야 합니다:

1. **마이그레이션 추가 시**:
   - `마이그레이션이력.md`에 새 항목 추가
   - Breaking Change 시 명시

2. **테이블 변경 시**:
   - `테이블상세.md` 업데이트
   - `스키마개요.md` ERD 수정

3. **인덱스 추가 시**:
   - `인덱스전략.md`에 문서화
   - 성능 측정 결과 포함

4. **새로운 쿼리 패턴 시**:
   - `쿼리패턴.md`에 예시 추가
   - SQLx 타입 매핑 설명

---

## 연락처

문서 관련 질문이나 개선 제안:
- GitHub Issues
- 프로젝트 메인테이너

---

**최종 업데이트**: 2025-01-15
**문서 버전**: 1.0
**스키마 버전**: 46개 마이그레이션 적용 완료
