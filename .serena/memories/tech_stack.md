# Tech Stack

## Backend
- **Language**: Rust (latest stable)
- **Web Framework**: Axum (async HTTP server)
- **Async Runtime**: Tokio
- **Database**: SQLite
- **ORM**: SQLx (compile-time checked queries)
- **Migrations**: SQLx migrations (timestamp-based, immutable)

## Frontend
- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite 6
- **UI Library**: shadcn/ui + Radix UI primitives
- **Styling**: Tailwind CSS
- **State Management**:
  - TanStack Query (server state)
  - Zustand (UI state)
  - React Context (auth, theme, etc.)
- **Router**: React Router v6
- **Testing**: Vitest + Testing Library
- **i18n**: i18next + react-i18next

## Type Sharing
- **ts-rs**: Auto-generates TypeScript types from Rust structs
- **Location**: `shared/types.ts` (auto-generated, DO NOT EDIT)

## Additional Libraries
- **Terminal**: xterm.js + @xterm/addon-fit
- **Code Editor**: CodeMirror 6 (@uiw/react-codemirror)
- **Rich Text**: Lexical
- **Drag & Drop**: @dnd-kit
- **WebSocket**: react-use-websocket
- **Virtualization**: react-virtuoso, react-window
- **Diff View**: @git-diff-view/react
- **Animation**: Framer Motion

## Development Tools
- **Package Manager**: pnpm (>=8)
- **Linting**: 
  - ESLint (frontend, max 50 warnings)
  - Clippy (Rust)
- **Formatting**:
  - Prettier (frontend)
  - rustfmt (backend)
- **Type Generation**: cargo run --bin generate_types

## Platform
- **OS**: Darwin (macOS), Linux support
- **Node**: >=18
- **Rust**: Latest stable (specified in rust-toolchain.toml)
