---
project_name: 'profile'
user_name: 'Riddler'
date: '2025-12-12'
sections_completed: ['technology_stack', 'language_rules', 'framework_rules', 'testing_rules', 'code_quality_rules', 'development_workflow_rules', 'anti_patterns']
status: 'complete'
rule_count: 62
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

### Core Technologies
- **Full-stack framework:** Leptos (compatible with Axum 0.8.x)
- **Web server:** Axum 0.8.x
- **Database:** SQLite 3.51.1 embedded
- **Database ORM:** `sqlx` 0.8.6 (with SQLite feature)
- **Password hashing:** `argon2` 0.5.3 with Argon2id parameters
- **Rate limiting:** `tower-governor` 0.8.0 with secure preset
- **CORS middleware:** `tower-http::cors` 0.6.8
- **UI components:** `leptix` (accessible unstyled components)
- **Styling:** Tailwind CSS
- **API documentation:** `utoipa` 5.4.0 for OpenAPI spec generation
- **Error handling:** `thiserror` crate (with unified `AppError` enum)
- **Build tool:** `cargo-leptos` (WASM optimization and hot reload)
- **Language:** Rust (full-stack type safety)

### Key Dependencies
- **Runtime:** Tokio (async runtime, version compatible with Axum 0.8.x)
- **Session management:** Custom database-backed sessions (no external crate)
- **Routing:** Leptos router (file-based routing from starter template)
- **Asset pipeline:** Dart-sass for SCSS compilation (starter template default)

## Critical Implementation Rules

### Language-Specific Rules (Rust + Leptos)

#### Error Handling
- **ALWAYS use `AppError` enum** for all error types - no `anyhow::Error`, `Box<dyn Error>`, or `String` errors
- **Implement `IntoResponse` for `AppError`** for consistent HTTP error responses
- **Leptos error boundaries** for catastrophic failures, local `Result` handling for recoverable errors
- **Structured JSON errors:** `{"error": {"code": "...", "message": "...", "details": {...}}}` format

#### Leptos Signal Management
- **Immutable updates:** `count.update(|c| *c + 1)` NOT `count.set(count.get() + 1)`
- **Derived signals:** Use `create_memo` for expensive computations, `create_selector` for equality checks
- **Resource patterns:** `create_resource` for async data with `Suspense` fallback UI
- **Component props:** Use custom types (`UserId`) not primitive types for type safety

#### Async & Concurrency
- **Tokio compatibility:** Ensure async runtime matches Axum 0.8.x requirements
- **No blocking calls:** Use `tokio::spawn` for CPU-bound work, never block async context
- **SQLx async queries:** Use `sqlx::query!` macros for compile-time SQL validation

#### Type Safety & Ownership
- **Leptos server functions:** Maintain full type safety across client/server boundary
- **Signal `Copy` trait:** Signals can be freely copied and moved into closures
- **Component architecture:** Feature-based organization with clear ownership boundaries

### Framework-Specific Rules (Leptos + Axum)

#### API Boundaries
- **Server functions ONLY for authenticated actions:** Login, registration, profile updates
- **REST routes ONLY for public data:** `GET /profiles/{username}` with `utoipa` documentation
- **Never mix patterns:** Authenticated endpoints = server functions, public endpoints = REST routes
- **OpenAPI documentation:** `utoipa` macros on all Axum routes, Rustdoc on server functions

#### Component Architecture
- **Base with `leptix`:** All components must use `leptix` unstyled components as foundation
- **Accessibility required:** Maintain ARIA support from `leptix` in all custom components
- **Tailwind styling:** Custom design via Tailwind utilities, no custom CSS files
- **Custom component patterns:** Profile URL Display, Action Cards, validation components

#### Routing & Authentication
- **File-based routing:** Follow starter template's `src/pages/` structure
- **Auth guards:** Route-level authentication middleware for protected routes
- **Session validation:** Server-side session checks, not client-side auth state
- **Public profile routes:** `GET /profiles/{username}` served via Axum, not Leptos

#### State Management
- **Local UI state:** Leptos signals for form inputs, toggles, filters
- **Server-side auth:** Session validation in middleware, no client auth state
- **Data resources:** `create_resource` with server functions for type-safe API calls
- **Form patterns:** Leptos signals with validation, consider `leptos_forms` for complex forms

#### Performance Rules
- **Automatic code splitting:** Trust `cargo-leptos` bundle optimization
- **Lazy loading:** Route-level code splitting via Leptos router
- **WASM optimization:** Production builds automatically use `wasm-opt`
- **Asset optimization:** Tailwind purging, image optimization via build pipeline

### Testing Rules

#### Test Structure
- **Unit tests:** `#[cfg(test)]` modules within source files - NOT separate test files
- **Integration tests:** `tests/` directory with feature-based organization (`auth.rs`, `profile.rs`, `api.rs`)
- **End-to-end tests:** `tests/e2e/` with Playwright for browser automation
- **No fixture directories:** Test data factories within test modules, not separate files

#### Test Organization
- **Feature-based testing:** Mirror source code structure (`tests/auth.rs` for `src/auth/`)
- **Public API tests:** `tests/api.rs` for Axum REST route testing with `utoipa` documentation
- **Authentication tests:** `tests/auth.rs` covering login, registration, session management
- **Profile tests:** `tests/profile.rs` covering profile CRUD and public profile access

#### Testing Framework
- **Cargo test standard:** Use Rust's built-in test framework, no external test runners
- **Playwright E2E:** Browser automation for critical user journeys (login → profile edit → share)
- **Trait-based testing:** Dependency injection via traits for testability, not mocking frameworks
- **SQLite in-memory:** `:memory:` database connections for isolated test execution

#### Coverage Requirements
- **SQL validation:** All database queries must use `sqlx::query!` macros for compile-time checking
- **Error case testing:** Test all `AppError` variants and HTTP error responses
- **Authentication flows:** Session creation/validation, rate limiting, password hashing
- **Type safety:** Verify Leptos server functions maintain type safety in test contexts

#### Test Data & Isolation
- **Self-contained tests:** Each test responsible for its own data setup/teardown
- **Database transactions:** Use transaction rollback or `:memory:` databases for isolation
- **No shared state:** Tests must not depend on execution order or shared global state
- **Clean test environment:** Ensure complete cleanup after each test execution

### Code Quality & Style Rules

#### Naming Conventions (MUST FOLLOW)
- **Database:** snake_case tables (plural) - `users`, `profiles`, `sessions`
- **Database columns:** snake_case - `user_id`, `username`, `created_at`
- **Database indexes:** `idx_table_column` pattern - `idx_users_email`
- **REST routes:** Plural resource names - `/profiles/{username}`, `/profiles/{username}/avatar`
- **Route parameters:** snake_case in `{}` - `{username}`, `{user_id}`
- **Header names:** kebab-case - `X-Request-Id`, `Content-Type`
- **Structs/Enums/Traits:** PascalCase - `UserProfile`, `AppError`, `DatabaseConnection`
- **Variables/Functions:** snake_case - `user_profile`, `get_user_data`, `validate_password`
- **Module/File names:** snake_case - `user_profile.rs`, `auth/mod.rs`
- **Constants:** SCREAMING_SNAKE_CASE - `MAX_LOGIN_ATTEMPTS`, `SESSION_DURATION_SECONDS`
- **Leptos Components:** PascalCase - `UserProfileCard`, `LoginForm`
- **Type parameters:** Descriptive - `UserId`, `ProfileData` (not just `T`, `U`)

#### Structure & Organization
- **Feature-based directories:** `auth/`, `profile/`, `shared/` - no mixing of unrelated code
- **Server functions:** Separate `server.rs` file within each feature directory
- **Types:** Feature-specific types in `types.rs`, cross-feature types in `shared/types.rs`
- **Tests:** Unit tests in `#[cfg(test)]` modules, integration tests in `tests/` directory
- **Assets:** `assets/` directory (required by cargo-leptos), `migrations/` for SQLx schemas

#### Format & Documentation
- **API success responses:** Direct data return - `{"id": 1, "username": "alice"}`
- **API error responses:** Structured error object - `{"error": {"code": "...", "message": "...", "details": {...}}}`
- **JSON field naming:** snake_case matching Rust structs - `user_id`, `created_at`
- **Date/time format:** ISO 8601 strings - `"2025-12-12T10:30:00Z"`
- **Documentation:** Rustdoc comments on server functions, `utoipa` macros on REST routes
- **OpenAPI spec:** Generated at `/api-docs/openapi.json` with Swagger UI optional

#### Enforcement Rules
- **ALWAYS validate patterns:** Check against this document before submitting code
- **NO deviations:** Follow naming conventions exactly - no exceptions
- **Code review priority:** Pattern compliance is highest review priority
- **Immediate fixes:** Fix pattern violations immediately, don't accumulate technical debt
- **Document updates:** Update this context file when new patterns are established

### Development Workflow Rules

#### Git & Repository
- **Branch naming:** Feature-based - `feat/auth-login`, `fix/profile-validation`, `docs/api-spec`
- **Commit messages:** Conventional commits - `feat: add user authentication`, `fix: resolve session expiry bug`
- **PR requirements:** Must include tests, follow established patterns, update docs if needed
- **Code review priority:** 1) Pattern compliance, 2) Functionality, 3) Performance

#### Development Commands
- **Local server:** `cargo leptos watch` - starts backend + frontend with hot reload
- **Database migrations:** `sqlx-cli` required - `sqlx migrate run` to apply migrations
- **Testing:** `cargo test` for unit/integration, Playwright for E2E (`npm test` in `tests/e2e/`)
- **Production build:** `cargo leptos build --release` with WASM optimization (`wasm-opt`)

#### Deployment & Environment
- **12-factor configuration:** Environment variables for ALL configuration
- **Required env vars:** `DATABASE_URL`, `SECRET_KEY`, `CORS_ORIGIN`, `RATE_LIMIT_REQUESTS`
- **Secrets:** Never commit secrets - use `.env` locally, platform secrets in production
- **Database:** SQLite file deployment - include in binary or use separate volume
- **Hosting:** Platform-agnostic binary - Vercel, Railway, Render, Fly.io, or VPS

#### CI/CD Pipeline
- **Build phase:** `cargo leptos build --release` with optimization flags
- **Test phase:** `cargo test` (unit/integration) + Playwright E2E in CI environment
- **Database for CI:** SQLite `:memory:` database for test isolation
- **Deployment validation:** Health check endpoint, environment validation

#### Environment Setup
- **Local development:** `.env` file with required variables (excluded from git)
- **Template:** `.env.example` with example values and documentation
- **Production:** Platform-specific secret management (no `.env` files in production)
- **Configuration loading:** `src/shared/config.rs` loads and validates all environment variables

#### Code Quality Workflow
- **Pre-commit:** No automated hooks initially, manual pattern validation required
- **PR checklist:** Tests passing, patterns followed, documentation updated
- **Deployment checklist:** Environment variables set, database migrated, health checks passing

### Critical Don't-Miss Rules

#### Security Gotchas
- **Password hashing:** Argon2id with parameters tuned for ~2 second verification (NFR P2)
- **Session cookies:** HTTP-only with `SameSite=Strict`, 24-hour expiry (NFR S3)
- **Rate limiting:** 5 failed attempts/minute per IP AND username (NFR S4)
- **CORS policy:** Frontend origin only, not `*` wildcard
- **SQL injection:** ALWAYS use `sqlx::query!` macros, never string concatenation

#### Performance Anti-Patterns
- **NO blocking in async:** Never use `std::thread::sleep` or blocking I/O in async handlers
- **Loading states required:** Every async operation must have visual loading indicator
- **WASM optimization:** Trust `cargo-leptos` + `wasm-opt`, don't manually split bundles
- **Database connections:** Use connection pooling, don't open/close per request

#### Common Implementation Mistakes
- **❌ Mixed naming:** `CREATE TABLE Users (userId INTEGER, created_at TEXT)` - WRONG
- **✅ Correct naming:** `CREATE TABLE users (user_id INTEGER, created_at TEXT)` - RIGHT
- **❌ String errors:** `Result<User, String>` or `Result<User, anyhow::Error>` - WRONG
- **✅ AppError only:** `Result<User, AppError>` with `IntoResponse` - RIGHT
- **❌ Primitive types:** `fn get_user(id: i32)` - WRONG
- **✅ Custom types:** `fn get_user(id: UserId)` - RIGHT

#### Edge Cases
- **Username validation:** Unicode characters, 3-30 characters, no reserved names
- **URL slug generation:** URL-safe, readable, handle collisions (append numbers)
- **Session expiry:** Clear UI message "Session expired, please log in again"
- **Rate limit UX:** "Too many attempts, try again in X minutes" not technical error

#### Framework Pitfalls
- **❌ Signal mutation:** `count.set(count.get() + 1)` - WRONG
- **✅ Signal update:** `count.update(|c| *c + 1)` - RIGHT
- **❌ Blocking component:** `let user = api_get_user().await` in component - WRONG
- **✅ Resource pattern:** `create_resource` with `Suspense` fallback - RIGHT
- **❌ Mixed API patterns:** Server function for public data - WRONG
- **✅ Clear boundaries:** Server functions = authenticated, REST routes = public - RIGHT

#### Testing Must-Haves
- **SQL compile-time checking:** All tests must use `sqlx::query!` macros
- **Error case coverage:** Test ALL `AppError` variants, not just happy path
- **Authentication flows:** Test session creation, validation, expiry, revocation
- **Rate limit testing:** Verify 5th attempt fails, 6th after minute succeeds

---

## Usage Guidelines

**For AI Agents:**

- Read this file before implementing any code
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Update this file if new patterns emerge

**For Humans:**

- Keep this file lean and focused on agent needs
- Update when technology stack changes
- Review quarterly for outdated rules
- Remove rules that become obvious over time

Last Updated: 2025-12-12