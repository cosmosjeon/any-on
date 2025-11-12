# .claude Directory

**Documentation and rules for Claude Code and developers working on this project.**

---

## 📁 Structure

```
.claude/
├── README.md              # This file
├── rules/                 # Development rules and architecture guidelines
│   ├── 01-critical-rules.md   # MUST-FOLLOW rules (type sync, migrations)
│   ├── 02-frontend.md         # Frontend architecture (React, TypeScript)
│   ├── 03-backend.md          # Backend architecture (Rust) [TBD]
│   └── 04-code-quality.md     # TDD, Clean Code, best practices
└── workflows/             # Common workflows and commands
    ├── commands.md            # Essential commands reference
    └── development.md         # Development workflows [TBD]
```

---

## 🚀 Quick Start for New Contributors

1. **Read critical rules first**: `.claude/rules/01-critical-rules.md`
   - Type synchronization (Rust → TypeScript)
   - Database migration rules
   - Development workflow

2. **Check architecture for your area**:
   - Frontend work? → `.claude/rules/02-frontend.md`
   - Backend work? → `.claude/rules/03-backend.md` (coming soon)

3. **Follow code quality standards**: `.claude/rules/04-code-quality.md`
   - TDD process
   - Clean Code principles
   - Naming conventions

4. **Reference commands**: `.claude/workflows/commands.md`
   - Common commands
   - Troubleshooting
   - Workflows

---

## 📖 When to Read Each Document

### Before Writing ANY Code
→ `.claude/rules/01-critical-rules.md`
- Understand type synchronization
- Learn migration rules
- Know the workflow

### Before Writing Frontend Code
→ `.claude/rules/02-frontend.md`
- Component structure
- State management patterns
- API client usage
- Testing approach

### Before Writing Backend Code
→ `.claude/rules/03-backend.md` [TBD]
- API structure
- Database patterns
- Error handling

### When Reviewing Code
→ `.claude/rules/04-code-quality.md`
- TDD checklist
- Clean Code principles
- Common anti-patterns

### When Stuck
→ `.claude/workflows/commands.md`
- Find the right command
- Troubleshooting tips
- Common workflows

---

## 🔍 Finding What You Need

### "How do I...?"

- **...add a new API endpoint?** → `.claude/workflows/commands.md` (Common Workflows)
- **...modify the database?** → `.claude/rules/01-critical-rules.md` (Rule #2)
- **...use state management?** → `.claude/rules/02-frontend.md` (Rule #4-7)
- **...write tests?** → `.claude/rules/04-code-quality.md` (TDD section)
- **...handle errors?** → `.claude/rules/04-code-quality.md` (Error Handling)
- **...name variables?** → `.claude/rules/04-code-quality.md` (Naming Conventions)

### "What's the rule for...?"

- **Type synchronization?** → `.claude/rules/01-critical-rules.md` (Rule #1)
- **Database migrations?** → `.claude/rules/01-critical-rules.md` (Rule #2)
- **Component size?** → `.claude/rules/02-frontend.md` (Rule #3)
- **Any types?** → `.claude/rules/02-frontend.md` (Rule #1)
- **Function length?** → `.claude/rules/04-code-quality.md` (Functions & Methods)

---

## 🤖 For Claude Code

When working on this project:

1. **Always check critical rules first** (`.claude/rules/01-critical-rules.md`)
2. **Follow architecture for the area you're modifying**:
   - Frontend → `.claude/rules/02-frontend.md`
   - Backend → `.claude/rules/03-backend.md`
3. **Maintain code quality standards** (`.claude/rules/04-code-quality.md`)
4. **Use correct commands** (`.claude/workflows/commands.md`)

### Priority Order

1. 🚨 Critical Rules (violations break builds)
2. 🏗️ Architecture Rules (maintain consistency)
3. ✨ Code Quality (maintainability)
4. 📝 Documentation (clarity)

---

## 📝 Document Maintenance

### When to Update

**Critical Rules** - When:
- Type generation process changes
- Migration strategy changes
- Core workflow changes

**Architecture Rules** - When:
- New patterns are established
- Technology choices change
- Best practices evolve

**Code Quality** - When:
- Team agrees on new standards
- New tools are adopted
- Lessons learned from issues

**Commands** - When:
- New commands are added
- Tool versions change
- Workflows are optimized

### How to Update

1. Edit the relevant `.md` file in `.claude/`
2. Ensure examples are accurate
3. Update `CLAUDE.md` if overview needs changes
4. Commit with: `docs: update <area> rules`

---

## 🔗 Related Files

- `/CLAUDE.md` - Main overview and quick reference
- `/README.md` - Project README for users
- `/frontend/package.json` - Frontend scripts
- `/Cargo.toml` - Backend configuration

---

**Last Updated:** 2025-11-12
**Maintainers:** Engineering Team
