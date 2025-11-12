# .claude/ - Claude Code 전용 문서

**⚠️ 이 폴더는 Claude Code와 AI 어시스턴트만을 위한 문서입니다.**

일반 개발자 문서는 `/docs` 폴더를 참조하세요.

---

## 📁 구조

```
.claude/
├── README.md                        # 이 파일
├── CRITICAL-RULES.md                # 🚨 필수 개발 룰
├── architecture/
│   ├── frontend.md                  # 프론트엔드 아키텍처 (33 rules)
│   ├── backend.md                   # 백엔드 아키텍처 [TBD]
│   └── code-quality.md              # TDD, Clean Code 원칙
└── guides/
    ├── commands.md                  # 자주 쓰는 명령어
    ├── frontend-first.md            # 프론트엔드 우선 개발 가이드
    └── development.md               # 일반 개발 워크플로우 [TBD]
```

---

## 🤖 For Claude Code

이 디렉토리의 모든 문서는 코드 작성 시 참조해야 하는 룰과 가이드입니다.

**우선순위:**
1. 🚨 `CRITICAL-RULES.md` - 반드시 따라야 함
2. 🏗️ `architecture/` - 해당 영역 작업 시 참조
3. 📖 `guides/` - 필요 시 참조

---

## 👨‍💻 For Human Developers

이 폴더는 AI 어시스턴트가 코드를 작성할 때 참조하는 룰입니다.

**사람이 읽을 문서:**
- 프로젝트 README: `/README.md`
- 사용자 문서: `/docs` (Mintlify)
- API 문서: [별도 위치]

**CLAUDE.md vs .claude/ 차이:**
- `CLAUDE.md`: 간결한 인덱스 및 핵심 룰 요약
- `.claude/`: 상세한 룰, 가이드, 예제 코드

---

**Last Updated:** 2025-11-12
