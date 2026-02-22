# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Postman - HTTP Client application built with Tauri 2.0, SvelteKit frontend and Rust backend. A Postman-like API testing tool.

## Principles

1. **Lint before commit**: Always run `cargo fmt && cargo clippy` before committing
2. **Type-safe errors**: Use `thiserror` for custom error types in Rust
3. **Keep CLAUDE.md updated**: Update this file when adding new features or patterns
4. **KISS**: Keep code simple, avoid unnecessary abstraction

## Build Commands

```bash
# Development
bun run tauri dev          # Run app in dev mode

# Frontend only
bun run dev                # Vite dev server
bun run check              # TypeScript type check

# Backend only
cd src-tauri
cargo check                # Check compilation
cargo fmt                  # Format code
cargo clippy               # Lint code

# Production
bun run tauri build        # Build release
```

## Project Structure

```
/src                          # Frontend (SvelteKit)
├── routes/
│   ├── +page.svelte         # Main HTTP Client UI
│   └── +layout.svelte       # App layout
├── lib/
│   └── bindings.ts          # Auto-generated (DO NOT EDIT)

/src-tauri                    # Backend (Rust)
├── src/
│   ├── lib.rs               # App initialization, state management
│   ├── main.rs              # Entry point
│   ├── modules/
│   │   ├── types.rs         # Shared types and events
│   │   └── logger.rs        # Logging utility
│   └── tools/
│       └── api_client/      # HTTP client implementation
│           ├── mod.rs
│           └── http_client.rs
└── Cargo.toml
```

## Features

### HTTP Client
- HTTP methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- Headers editor with enable/disable toggle
- Request body input (JSON)
- Response viewer (status, time, headers, body)
- Auto JSON formatting
- Templates: Save/load/delete request templates
- History: Last 50 requests with quick reload
- Data persistence via tauri-plugin-store

## TypeScript/Svelte Guidelines

### Imports
```typescript
import { onMount } from "svelte"
import { commands } from "$lib/bindings"
import { Store } from "@tauri-apps/plugin-store"
```

### Svelte 5 Runes
```typescript
let count = $state(0)
let doubled = $derived(count * 2)
```

### Formatting
- No semicolons
- Double quotes for strings
- 2 spaces indentation

## Rust Guidelines

### Tauri Commands
```rust
#[tauri::command]
#[specta::specta]
pub async fn send_http_request(request: HttpRequest) -> Result<HttpResponse, HttpClientError> {
    // implementation
}
```

After creating a command:
1. Add to `collect_commands![]` in `lib.rs`
2. Run app to regenerate TypeScript bindings

### Error Handling
Use thiserror for typed errors:
```rust
#[derive(Debug, thiserror::Error, specta::Type, serde::Serialize)]
pub enum HttpClientError {
    #[error("Request failed: {0}")]
    RequestError(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}
```

### Types for Frontend
```rust
#[derive(Serialize, Deserialize, specta::Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub body: String,
}
```

### Naming Conventions
| Type | Convention | Example |
|------|------------|---------|
| Files | snake_case | `http_client.rs` |
| Functions | snake_case | `send_request()` |
| Types/Structs | PascalCase | `HttpRequest` |
| Constants | UPPER_SNAKE | `MAX_HISTORY` |

## Before Committing

Husky pre-commit hook automatically runs:
- Git user.name/email verification
- `cargo fmt --check` and `cargo clippy`
- `bun run check` for TypeScript validation

Manual checks (if needed):
```bash
cd src-tauri && cargo fmt && cargo clippy
bun run check
```

## Release Process

Releases are automated via GitHub Actions when you push a tag:

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

This triggers `.github/workflows/release.yml` which:
1. Builds for Windows (exe), macOS (dmg/app - Universal binary), Linux (AppImage/deb)
2. Uses Rust cache for faster builds
3. Creates a draft release with all platform binaries
4. Review and publish the draft release on GitHub

## Common Tasks

### Adding a New Command
1. Define in `src-tauri/src/tools/api_client/` with `#[tauri::command]` + `#[specta::specta]`
2. Add to `collect_commands![]` in `lib.rs`
3. Run `bun run tauri dev` to regenerate bindings
4. Use via `commands.myCommand()` in frontend

### Data Storage
Templates and history are stored using tauri-plugin-store:
```typescript
const store = await Store.load("http-client.json")
await store.set("templates", templates)
await store.set("history", history)
await store.save()
```

## Keeping This File Updated

When you add:
- New command -> Document pattern if non-trivial
- New module -> Add to project structure
- New dependency -> Note if it changes patterns
- New convention -> Add to guidelines
