# Scripts Directory

This directory contains utility scripts for Anyon development.

## Demo Data Setup

### 📋 Overview

The demo data setup provides a pre-populated database with realistic projects, epics, and stories for UI development and testing.

### 🚀 Quick Start

```bash
# From project root
./scripts/apply-demo-data.sh
```

### 📊 What's Included

The demo data includes:

#### Projects (3)
1. **Demo Project - E-Commerce** 
   - Shopping cart, user auth, product catalog
2. **Demo Project - API Service**
   - RESTful API implementation
3. **Demo Project - Mobile App**
   - Onboarding flow

#### Epic-Story Structure

Each project contains **Epics** (high-level features) with nested **Stories** (specific tasks):

**E-Commerce Project:**
- **Epic: User Authentication System** _(in progress)_
  - ✅ Story: Email/Password Login _(done)_
  - ✅ Story: Social Login (Google, GitHub) _(done)_
  - 🔄 Story: Password Reset Flow _(in progress)_
  - 📋 Story: JWT Token Management _(todo)_
  - 📋 Story: 2FA Implementation _(todo)_

- **Epic: Shopping Cart Features** _(todo)_
  - 📋 Story: Add/Remove Items
  - 📋 Story: Cart Persistence
  - 📋 Story: Price Calculation
  - 📋 Story: Checkout Integration

- **Epic: Product Catalog Management** _(in review)_
  - ✅ Story: Product List View _(done)_
  - 🔍 Story: Advanced Search _(in review)_
  - ✅ Story: Product Detail Page _(done)_

Plus standalone tasks like bug fixes and maintenance work.

### 🎯 Usage

1. **Apply demo data:**
   ```bash
   ./scripts/apply-demo-data.sh
   ```

2. **Start the development server:**
   ```bash
   pnpm run dev
   ```

3. **View in Kanban board:**
   - Navigate to Kanban page
   - Select "Demo Project - E-Commerce"
   - Explore the Backlog column
   - Click Epic chevrons to expand/collapse stories

### 🔑 Key Features

- **Epic-Story Hierarchy**: Tasks prefixed with "Epic:" or "Story:" are grouped hierarchically in the Backlog column
- **Various States**: Tasks demonstrate all workflow states (todo, inprogress, inreview, done, cancelled)
- **Orphaned Data**: All demo data has `user_id = NULL`, making it "orphaned"
- **Auto-Claim**: First GitHub login automatically claims orphaned data

### 🗄️ Database Details

- **File**: `seed-demo-data.sql`
- **Tables**: `projects`, `tasks`
- **User ID**: `NULL` (orphaned)
- **UUIDs**: Fixed UUIDs for reproducibility

### 🔄 Resetting Demo Data

To reset and reapply:

```bash
# Remove current data
sqlite3 anyon.db "DELETE FROM tasks WHERE user_id IS NULL"
sqlite3 anyon.db "DELETE FROM projects WHERE user_id IS NULL"

# Reapply
./scripts/apply-demo-data.sh
```

### ⚠️ Important Notes

1. **Backup**: The script automatically creates a timestamped backup before applying changes
2. **Development Only**: This is for local development and UI testing only
3. **First Login**: Demo data will be claimed by your first GitHub login
4. **Epic Detection**: Epics are identified by "Epic:" prefix in title (prototype implementation)

### 🐛 Troubleshooting

**Database not found:**
```bash
# Run from project root or scripts directory
cd /path/to/anyon
./scripts/apply-demo-data.sh
```

**sqlite3 not installed:**
```bash
# macOS
brew install sqlite3

# Ubuntu
sudo apt-get install sqlite3
```

**Permission denied:**
```bash
chmod +x scripts/apply-demo-data.sh
```

### 📚 Related Files

- `seed-demo-data.sql` - SQL seed data
- `apply-demo-data.sh` - Application script
- `frontend/src/components/tasks/EpicCard.tsx` - Epic UI component
- `frontend/src/components/tasks/TaskKanbanBoard.tsx` - Kanban board with Epic support

---

## Future Improvements

For production-ready Epic-Story functionality, consider:

1. **Backend Schema Changes**:
   ```rust
   pub enum TaskType { Epic, Story, Task }
   pub parent_task_id: Option<Uuid>
   ```

2. **Database Migration**:
   ```bash
   sqlx migrate add add_task_hierarchy
   ```

3. **Type Regeneration**:
   ```bash
   npm run generate-types
   ```

This would replace the current title-based detection with proper database relationships.
