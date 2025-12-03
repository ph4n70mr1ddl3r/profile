---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - docs/prd.md
  - docs/ux-design-specification.md
  - docs/analysis/research/market-creator-brand-fan-platform-research-2025-12-03.md
workflowType: 'architecture'
lastStep: 8
project_name: 'opencode'
user_name: 'Riddler'
date: '2025-12-03T16:33:08+08:00'
status: 'complete'
completedAt: '2025-12-03T16:33:08+08:00'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
- Verification-first platform: creator/fan verification with authenticity score/badge, recovery paths, never show unverified as verified.
- Brief→shortlist→intro→campaign→report loop: automated verified shortlists with partial/warning fallback, intro approvals, campaign tracking, reports with signals/flags/audit trail, exports/webhooks.
- Multi-tenant + RBAC: shared tenancy with strict scoping; roles for Brand Admin/Member, Creator, Fan, Ops/Admin; plan/tier gates and usage limits.
- Trust UX + degraded mode: badges/signals/warnings inline across profiles/shortlists/reports; explicit degraded mode when signal providers hiccup.
- Notifications/integrations: email/notifications for key events; webhooks/exports; plug-in model for signal providers.
- Ops controls: flag queue, overrides, rule updates, anomaly alerts, auditability.

**Non-Functional Requirements:**
- Performance/SLA: p95 <2s for verification/shortlist/report flows; shortlist result <24h; degraded mode surfaces warnings.
- Reliability/availability: 99.9% target for verification/shortlist services; guardrails to avoid false verification; audit trail durability.
- Security/privacy: tenant isolation, encrypted data, least-privilege, scoped access to signals; no regulated data expected.
- Scalability/extensibility: low-thousands concurrent sessions; 10x shortlist/report volume with <10% perf degradation; plug-in signal providers.
- Accessibility/UX quality: WCAG 2.1 AA, focus states, icon+label for trust states; responsive desktop-first with mobile/tablet support.
- Observability/ops: monitoring for throughput/SLA/degraded events/anomalies; auditable rule changes and revert paths.

**Scale & Complexity:**
- Primary domain: web app (saas_b2b) with APIs/services for verification, matching, reporting.
- Complexity level: medium-high (multi-tenant RBAC, verification pipelines, plug-in signals, degraded-mode correctness, reporting/export).
- Estimated architectural components: ~8–10 (auth/identity + RBAC/tenancy, verification service, signal provider adapters, matching/shortlist service, reporting/export service, notifications/webhooks, web app/frontends, ops/flag queue, audit/observability).

### Technical Constraints & Dependencies
- Must enforce tenant scoping everywhere; multi-tenant shared baseline.
- Performance guardrails: p95 <2s on core flows; shortlist SLA <24h.
- Availability: 99.9% for verification/shortlist; degraded mode required when providers fail.
- Data integrity: never display unverified as verified; audit trail for signals/overrides/reports.
- UX constraints: trust cues inline with badges/signals/warnings; WCAG AA; responsive desktop-first.
- Plug-in model for signal providers; explicit handling for partial/failed signals.
- No regulated data expected; baseline privacy and encryption.

### Cross-Cutting Concerns Identified
- Tenant isolation and RBAC across all services and data paths.
- Trust UX surfaces: badges/signals/warnings across profiles, shortlists, reports, and degraded mode.
- Auditability and observability: logs/traces/metrics for verification, shortlist SLA, degraded events, rule changes.
- Notifications/webhooks/export consistency with permissions and degraded/partial annotations.
- Fallback/degraded behaviors when signal providers fail; recovery loops for verification.
- Usage/plan enforcement tied to RBAC and billing context.

## Starter Template Evaluation

### Primary Technology Domain
Full-stack web app (Next.js + TypeScript) based on SaaS B2B requirements and trust-focused UX.

### Starter Options Considered
- Next.js App Router + TypeScript + Tailwind: strong SSR/ISR, flexible API routes, broad ecosystem, good DX; easy to layer Prisma/Postgres and testing (Vitest/Playwright).
- Remix: great data mutations and forms, but less standard for dashboard SSR/ISR at scale here.
- Vite SPA: fast dev, but would require separate API/backends and more wiring for auth/SSR/reporting.

### Selected Starter: Next.js (App Router) with TypeScript and Tailwind
**Rationale for Selection:**
- Matches web SaaS with dashboards/reports and trust UX; SSR/ISR for speed, SEO for marketing.
- App Router supports layouts/streaming; easy to add auth, RBAC guards, and middleware for tenant scoping.
- Tailwind for fast, consistent UI with accessible patterns; can layer a headless component kit later.
- Ecosystem support for Postgres (Prisma), testing (Vitest/Playwright), linting/formatting built-in.

**Initialization Command:**
```bash
npx create-next-app@latest \
  --ts \
  --tailwind \
  --eslint \
  --app \
  --src-dir \
  --use-npm \
  --import-alias "@/*"
```

**Architectural Decisions Provided by Starter:**
- **Language & Runtime:** TypeScript, React 18+, Next.js App Router.
- **Styling Solution:** Tailwind prewired; easy to add headless UI components and semantic tokens for trust states.
- **Build Tooling:** Next build/serve, SWC/Next compiler; image/fonts optimization; ready for ISR/SSG/SSR.
- **Testing Framework:** None by default—plan to add Vitest + React Testing Library and Playwright for E2E.
- **Code Organization:** App Router with nested layouts; `src/` alias `@/*`; API routes in `app/api/*`; supports route groups for RBAC/tenant scoping.
- **Development Experience:** Hot reload, ESLint, TypeScript strict defaults; ready for Vercel or similar deployment.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Postgres + Prisma for multi-tenant data with row-level scoping (tenant_id on every domain table); Prisma Migrate for schema management.
- Auth via NextAuth (App Router) with Postgres adapter; tenant-aware sessions; role-based and tenant-scoped authorization enforced in middleware + data layer.
- API pattern: Next.js API routes (REST-ish) with Zod schemas; OpenAPI doc generation; consistent error envelope.
~- Infra baseline: Vercel deploy; managed Postgres (Neon/Supabase); managed Redis (Upstash) for sessions/rate limits/cache; GitHub Actions CI.~
- Infra baseline: Vercel deploy; managed Postgres (Neon/Supabase); managed Redis (Upstash) for sessions/rate limits/cache; GitHub Actions CI.
- Observability: Sentry for app errors + Axiom/Logtail (or Vercel logs) for structured logs; health/SLA metrics for verification/shortlist flows.

**Important Decisions (Shape Architecture):**
- Data validation: Zod at edges (API/routes/forms) + Prisma schema constraints; audit tables for verification signals, overrides, and report annotations.
- Caching: Redis for session/storefront tokens, shortlist/report partials, and rate limiting; ISR for marketing; server data fetching with SWR/React Query for client interactivity.
- Authorization model: role/tenant checks via middleware + service-layer guards; policy helpers for “never show unverified as verified”.
- Frontend state: React Server Components where possible; TanStack Query for client data fetching/mutations; lightweight local state (Zustand/Context) if needed.
- Testing: add Vitest + React Testing Library for units; Playwright for E2E on critical flows (verification, shortlist, degraded mode).

**Deferred Decisions (Post-MVP):**
- Full-blown event bus/queues (Kafka/SNS/SQS) for signals; start with in-process + webhooks and graduate to queues later.
- Feature flag service (e.g., LaunchDarkly/Flagsmith); start with env-driven toggles.
- Multi-region deployment; begin single-region with clear isolation and back up plans.

### Data Architecture
- DB: Postgres (managed Neon/Supabase). Verify latest server version; assume ≥14. Enforce tenant_id on all tables; composite unique keys include tenant_id.
- ORM/Migrations: Prisma with Prisma Migrate; seed scripts per environment; generate Zod types from Prisma if desired.
- Validation: Zod on input; DB constraints for critical invariants (e.g., no unverified-as-verified flags).
- Auditing: append-only audit tables for verification signals, overrides, report annotations.
- Caching: Redis for sessions/rate limits and small response caches (shortlist/report summaries); time-bounded and tenant-scoped.

### Authentication & Security
- Auth: NextAuth with credentials + OAuth (and passkeys later); Postgres adapter; session tokens stored in cookies; optional Redis session cache.
- Authorization: role + tenant checks in middleware (protects app routes) and service layer (protects data access). Policy helpers for “never show unverified as verified”.
- Security middleware: Next.js middleware for auth/tenant gating; CSP, HTTPS, HSTS, rate limiting via Redis.
- Secrets: Vercel env for secrets; DB encryption at rest (provider); TLS in transit; Argon2 for password hashing if used.

### API & Communication Patterns
- API style: REST-ish via Next.js API routes; Zod for request/response schemas; OpenAPI doc generation from Zod.
- Error envelope: structured `{error: {code, message, details}}`; HTTP semantics aligned.
- Webhooks: for shortlist ready, campaign status changes; signed with shared secret; replay protection via nonce + TTL stored in Redis/DB.
- Rate limiting: Redis-based sliding window keyed by user/tenant/IP on auth endpoints, shortlist/report fetches, and webhooks.
- Real-time: defer sockets; start with polling/SWR revalidation; add Pusher/Ably/Supabase Realtime later if needed.

### Frontend Architecture
- Framework: Next.js App Router with TypeScript; RSC-first; client components for forms/interactions.
- State/data fetching: TanStack Query for mutations/live-ish data; form validation with Zod + React Hook Form.
- Styling/UI: Tailwind; add headless UI kit for accessibility (e.g., Radix/HeadlessUI) with trust-state tokens (verified/warning/error).
- Performance: ISR/SSR for marketing; server data fetching for dashboards; image optimization; route groups for RBAC/tenant segmentation.

### Infrastructure & Deployment
- Hosting: Vercel (app + API routes); Postgres on Neon/Supabase; Redis on Upstash.
- CI/CD: GitHub Actions running lint, type-check, unit/E2E tests; deploy on main.
- Config: `.env` per environment; Vercel env for runtime secrets; keep Prisma shadow DB for safe migrations.
- Monitoring/Logging: Sentry for FE/BE errors; Axiom/Logtail or Vercel log drains for structured logs; uptime/health checks for verification/shortlist services.
- Scaling: start single region; vertical autoscale via managed services; plan sharding/queue offload when signal volumes grow.

### Decision Impact Analysis
- Implementation Sequence: (1) Set up repo with starter and env scaffolding → (2) Add Prisma/Postgres + migrations + seed → (3) Wire NextAuth + RBAC/tenant middleware → (4) Establish API schemas + error/rate limiting → (5) Add Redis caching/rate limit layer → (6) Integrate Sentry/logging → (7) Add TanStack Query + form validation patterns → (8) E2E tests for critical flows.
- Cross-Component Dependencies: Auth/RBAC depends on DB schema + middleware; rate limiting depends on Redis; auditability depends on DB schema + API envelope; trust UX depends on API surfacing signals/flags; webhooks depend on shared secret + replay protection.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:** naming (DB/API/code), structure (tests/components/services), formats (API envelopes, dates), communication (events/state), process (errors/loading/auth).

### Naming Patterns

**Database Naming Conventions:**
- Tables: snake_case plural, e.g., `creators`, `brand_briefs`, `shortlists`, `verification_signals`.
- Columns: snake_case; foreign keys `*_id`; timestamps `created_at`, `updated_at`; tenant scoping `tenant_id`; soft-delete (if added) `deleted_at`.
- Indexes: `idx_<table>_<cols>`; unique constraints `uniq_<table>_<cols>`.

**API Naming Conventions:**
- REST-ish paths plural, kebab: `/api/creators`, `/api/brand-briefs/{id}`, `/api/shortlists/{id}/reports`.
- Query params snake_case: `tenant_id`, `limit`, `cursor`.
- Headers: `X-Request-Id`, `X-Tenant-Id` (if not in session), `X-Api-Version`.

**Code Naming Conventions:**
- Files/components: kebab-case file names; PascalCase component names (`CreatorCard.tsx`), camelCase functions/variables (`fetchShortlist`).
- Route segments: kebab; dynamic routes `[id]`; route groups for RBAC/tenant grouping (e.g., `(app)/(brand)/briefs`).

### Structure Patterns

**Project Organization:**
- Feature-first under `src/app/(app)/`: group routes by domain (`briefs`, `shortlists`, `reports`, `verification`, `ops`).
- Shared libs in `src/lib` (auth, db, validation, telemetry, rbac policies), `src/server` for service-layer logic, `src/components` for shared UI; feature-scoped components under feature folders.
- Tests: co-locate unit/integration as `*.test.ts(x)` next to code; E2E in `e2e/` with Playwright fixtures per role/tenant.

**File Structure Patterns:**
- API route handlers in `src/app/api/.../route.ts` with request/response Zod schemas.
- Prisma schema in `/prisma/schema.prisma`; migrations in `/prisma/migrations`; seeds in `/prisma/seed.ts`.
- Config: env parsing in `src/config/env.ts` using Zod; single source of truth imported elsewhere.

### Format Patterns

**API Response Formats:**
- Success envelope: `{ data, meta? }`; Errors: `{ error: { code, message, details? } }`.
- Use HTTP semantics (201 create, 400/401/403/404/409/422/429/500); include `requestId` header and log correlation.
- Dates as ISO 8601 strings in JSON; store as timestamptz in DB.

**Data Exchange Formats:**
- JSON fields camelCase at API boundary; map to snake_case DB via Prisma mappings.
- Booleans as true/false; no 0/1; null when absent, avoid `undefined` in API responses.

### Communication Patterns

**Event System Patterns:**
- Names: dot notation, past tense, scoped: `verification.completed`, `shortlist.generated`, `report.published`, `webhook.received`.
- Payload: include `id`, `tenantId`, `actor`, `occurredAt`, `version`, and domain-specific body; reserve `meta` for trace ids.
- Version events with `version` field; avoid breaking changes—additive only.

**State Management Patterns:**
- Server Components default; client components only for interactivity/forms.
- Data fetching/mutations via TanStack Query; keys scoped by tenant and resource (`['shortlist', tenantId, id]`).
- Immutable updates; keep derived selectors in helpers; loading/error states driven by query status, not ad-hoc booleans.

### Process Patterns

**Error Handling Patterns:**
- API: throw typed errors mapped to envelope codes; log with requestId, tenantId, userId, role; redact PII.
- UI: error boundaries per route segment; user-facing messages concise, with retry where safe; degraded-mode warnings inline near affected data.
- Validation failures return 422 with field-level details.

**Loading State Patterns:**
- Use Suspense + skeletons for data views; button-level spinners for mutations; disable idempotent actions while pending.
- Keep optimistic updates scoped and reversible; show rollback on failure.

### Enforcement Guidelines

**All AI Agents MUST:**
- Apply tenant scoping (`tenant_id`) in every DB query and API handler; never return cross-tenant data.
- Use the standardized API envelopes and ISO dates; never return raw errors.
- Validate all inputs/outputs with Zod; keep schemas co-located with handlers/forms.

**Pattern Enforcement:**
- Lint/type-check/format in CI; schema-driven tests for API envelopes; Playwright smoke for auth/RBAC/tenant leakage; review PRs for policy helpers usage.
- Document violations in PR feedback; update patterns section when standards evolve (with rationale).

### Pattern Examples

**Good:**
- API handler parses `tenantId` from session, validates body with Zod, returns `{ data }` and sets `X-Request-Id`.
- Prisma query: `ctx.db.shortlist.findUnique({ where: { id_tenantId: { id, tenantId } } })`.
- Event payload: `{ version: 1, name: 'verification.completed', tenantId, actor: { id, role }, occurredAt, data: { verificationId, status } }`.

**Anti-Patterns:**
- Returning raw Prisma errors or stack traces to clients.
- Queries without tenant scoping or relying on client-provided tenantId without server-side assertion.
- Mixing snake_case and camelCase in API payloads or inconsistent error shapes.

## Project Structure & Boundaries

### Complete Project Directory Structure
opencode/
├── README.md
├── package.json
├── tsconfig.json
├── next.config.js
├── tailwind.config.js
├── postcss.config.js
├── .eslintrc.js
├── .gitignore
├── .env.example
├── .env.local            # not committed
├── .github/
│   └── workflows/
│       └── ci.yml
├── prisma/
│   ├── schema.prisma
│   ├── migrations/
│   └── seed.ts
├── public/
│   └── assets/           # logos, icons, static images
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   ├── globals.css
│   │   ├── (marketing)/
│   │   │   └── page.tsx
│   │   ├── (app)/
│   │   │   ├── layout.tsx           # authenticated shell
│   │   │   ├── page.tsx             # dashboard/home
│   │   │   ├── briefs/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [briefId]/page.tsx
│   │   │   ├── shortlists/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [shortlistId]/page.tsx
│   │   │   ├── reports/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [reportId]/page.tsx
│   │   │   ├── verification/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [verificationId]/page.tsx
│   │   │   ├── ops/
│   │   │   │   ├── page.tsx
│   │   │   │   └── flags/[flagId]/page.tsx
│   │   │   └── settings/
│   │   │       ├── page.tsx
│   │   │       └── billing/page.tsx
│   │   └── api/
│   │       ├── auth/[...nextauth]/route.ts
│   │       ├── briefs/route.ts
│   │       ├── briefs/[briefId]/route.ts
│   │       ├── shortlists/route.ts
│   │       ├── shortlists/[shortlistId]/route.ts
│   │       ├── reports/route.ts
│   │       ├── reports/[reportId]/route.ts
│   │       ├── verification/route.ts
│   │       ├── verification/[verificationId]/route.ts
│   │       ├── webhooks/[provider]/route.ts   # signed webhooks
│   │       └── health/route.ts
│   ├── components/
│   │   ├── ui/                 # buttons, inputs, modals, alerts, badges
│   │   ├── layout/             # nav, sidebar, topbar, shell pieces
│   │   ├── trust/              # badges/signals/warnings components
│   │   └── features/           # shared feature widgets (cards, tables)
│   ├── lib/
│   │   ├── db.ts               # Prisma client
│   │   ├── auth.ts             # NextAuth helpers, session, RBAC guards
│   │   ├── rbac.ts             # role/tenant policies, helper checks
│   │   ├── validation/         # Zod schemas (API + forms)
│   │   ├── api/                # API client helpers/fetchers
│   │   ├── telemetry.ts        # Sentry/logging hooks
│   │   ├── cache.ts            # Redis access helpers
│   │   ├── config/env.ts       # Zod env parsing
│   │   └── errors.ts           # typed errors -> API envelope mapping
│   ├── server/
│   │   ├── services/           # domain services (briefs, shortlists, reports, verification)
│   │   ├── repositories/       # data access wrappers around Prisma
│   │   ├── mappers/            # DB <-> API model mapping (snake/camel)
│   │   └── workflows/          # orchestrations (shortlist generation, verification flows)
│   ├── middleware.ts           # auth/tenant/rate-limit middleware
│   ├── styles/                 # Tailwind extensions, tokens (trust states)
│   ├── types/                  # shared types/interfaces
│   ├── hooks/                  # client hooks (e.g., useTenant, useTrustSignals)
│   └── utils/                  # small shared utilities
├── tests/
│   ├── e2e/                    # Playwright specs/fixtures per role/tenant
│   ├── integration/            # API route/service tests
│   └── unit/                   # lib/helpers/service unit tests
└── docs/
    └── architecture.md         # this doc; add project README, ADRs if needed

### Architectural Boundaries

**API Boundaries:**
- External: `/api/briefs`, `/api/shortlists`, `/api/reports`, `/api/verification`, `/api/webhooks/{provider}`, `/api/health`.
- Auth boundary: middleware enforces session + tenant; NextAuth routes isolated; webhook routes require signature + replay protection.
- Rate limiting boundary: Redis-backed on auth, shortlist/report fetches, webhooks.

**Component Boundaries:**
- UI separation: `components/ui` primitives; `components/trust` for badges/signals/warnings; feature pages own domain composition under `app/(app)/...`.
- Route groups: `(app)` authenticated shell; `(marketing)` public.
- State boundaries: server components first; client components for forms/interactivity; TanStack Query for mutations/fetches.

**Service Boundaries:**
- `server/services` per domain (briefs, shortlists, verification, reports, ops) calling repositories.
- Repositories encapsulate Prisma and tenant scoping; mappers handle DB↔API shape.
- Workflows orchestrate multi-step flows (verification pipeline, shortlist generation).

**Data Boundaries:**
- Tenant scoping in every repo call; composite keys include `tenant_id`.
- Prisma schema holds RBAC/role bindings; audit tables for verification signals/overrides/report annotations.
- Redis caches: sessions (optional), rate limits, shortlist/report partials; TTL-scoped per tenant.

### Requirements to Structure Mapping

**Briefs → Shortlists → Reports Loop**
- Pages: `src/app/(app)/briefs`, `.../shortlists`, `.../reports`
- API: `src/app/api/briefs`, `.../shortlists`, `.../reports`
- Services: `src/server/services/{briefs,shortlists,reports}`
- Data: `prisma/migrations/*` for briefs, shortlist entries, reports, audit tables
- Tests: `tests/integration` for API, `tests/e2e` for flows

**Verification (Creators/Fans)**
- Pages: `src/app/(app)/verification`
- API: `src/app/api/verification`
- Services/workflows: `src/server/services/verification`, `src/server/workflows/verification`
- Data: verification sessions/signals tables; audit trail
- Tests: integration + e2e for verification, degraded/partial states

**Ops/Admin (Flags/Overrides/Degraded Mode)**
- Pages: `src/app/(app)/ops`
- API: could share verification/reporting endpoints with elevated roles or dedicated routes
- Services: `src/server/services/ops`
- Data: ops actions, overrides, rule changes in audit tables
- Tests: e2e for ops actions, integration for rule updates

**Auth/RBAC/Tenancy**
- Middleware: `src/middleware.ts`
- Lib: `src/lib/auth.ts`, `src/lib/rbac.ts`, `src/lib/config/env.ts`
- Data: tenant/org tables, user-role bindings
- Tests: integration for middleware, e2e for role/tenant leakage checks

**Notifications/Webhooks**
- API: `src/app/api/webhooks/[provider]/route.ts`
- Lib: signing/verification helpers; rate limit + replay protection in Redis
- Services: handlers in `src/server/services`
- Tests: integration for signature/replay; e2e for webhook-driven updates

### Integration Points

**Internal Communication:**
- Pages/route handlers call services; services call repositories; mappers translate shapes; Zod validates at edges.
- Events (if added) emitted from services/workflows with namespacing; stored or dispatched via webhook/queue later.

**External Integrations:**
- Webhooks from signal providers → `api/webhooks/[provider]`
- Email/notifications (future) via provider client in `src/lib` with wrappers respecting tenant/rate limits.

**Data Flow:**
- Request → middleware (auth/tenant/rate limit) → route handler (Zod validate) → service → repo (tenant-scoped Prisma) → mapper → response envelope.

### File Organization Patterns

**Configuration Files:**
- Env schema `src/lib/config/env.ts`; `.env.example` documents required vars; Vercel env for secrets.
- Tailwind/Next/TS configs at root; ESLint config at root.

**Source Organization:**
- Feature-first under `app/(app)/...`; shared primitives in `components/ui`; domain UIs under feature routes; services/repositories/mappers split under `server/`.

**Test Organization:**
- Unit/integration co-located with `tests/unit` and `tests/integration`; E2E in `tests/e2e` with role/tenant fixtures.

**Asset Organization:**
- Static assets in `public/assets`; trust-state icons/badges here.

### Development Workflow Integration
- Dev server: Next.js `next dev`; uses `middleware.ts` for auth/tenant gating in dev.
- Build: `next build`; relies on Prisma generate/migrate; env validated at startup.
- Deployment: Vercel (app + API); managed Postgres/Redis; CI runs lint/typecheck/tests before deploy.

## Architecture Validation Results

### Coherence Validation ✅
**Decision Compatibility:** Stack components align (Next.js + Prisma/Postgres + Redis + NextAuth + Zod + TanStack Query + Vercel). No internal conflicts; REST-ish API + Zod + Prisma mapping consistent; Redis use matches rate limits/cache/session needs.
**Pattern Consistency:** Naming (snake_case DB, camelCase API), envelopes, tenant scoping, and error/loading patterns align with decisions. Structure matches feature-first App Router layout and service/repo layers.
**Structure Alignment:** Directory tree supports routes, services, repositories, mappers, and tests; middleware enforces auth/tenant/rate limits; docs and env parsing locations defined.

### Requirements Coverage Validation ✅
**Feature Coverage:** Brief→shortlist→report, verification, ops/admin, notifications/webhooks, and auth/RBAC/tenancy all mapped to pages, APIs, services, data, and tests.
**Functional Requirements:** FRs covered via services/routes/workflows and auditability; degraded/partial modes via patterns; exports/webhooks endpoints planned.
**Non-Functional Requirements:** Perf (p95<2s) via SSR/ISR, caching, rate limits; availability via managed Postgres/Redis + health checks; security via middleware + RBAC + audit; scalability/extensibility via plugin-able providers and event-ready workflows; accessibility via UI patterns and Tailwind/headless approach.

### Implementation Readiness Validation ✅
**Decision Completeness:** Critical choices locked; envelopes, validation, tenant scoping, error/rate limit policies defined.
**Structure Completeness:** Concrete tree with pages/APIs/services/repos/mappers/tests and config locations.
**Pattern Completeness:** Naming/format/process patterns set; examples and anti-patterns included.

### Gap Analysis Results
- Critical: None identified.
- Important: Need to lock exact versions (Next.js, Prisma, NextAuth, Redis client) once network/version check is available.
- Nice-to-Have: Add ADRs per major decision; document webhook signature scheme and replay policy; add feature flag plan.

### Validation Issues Addressed
- Version verification deferred until network check is possible; noted as follow-up.

### Architecture Completeness Checklist
- [x] Project context analyzed; scale/constraints mapped; cross-cutting concerns covered
- [x] Critical decisions + stack specified; integration and performance patterns defined
- [x] Naming/structure/communication/process patterns documented
- [x] Directory structure, boundaries, integration points, and requirement mapping defined

### Architecture Readiness Assessment
**Overall Status:** READY FOR IMPLEMENTATION  
**Confidence Level:** High (pending exact version pinning once verified)

**Key Strengths:** Tenant/RBAC guardrails; trust UX patterns; auditability; clear API envelopes; feature-first structure; testing layout with e2e focus on trust flows.  
**Areas for Future Enhancement:** Version pinning; ADRs; formal webhook signature policy and replay window; add feature flag strategy post-MVP.

### Implementation Handoff
**AI Agent Guidelines:** Follow documented envelopes, Zod validation, tenant scoping, and directory boundaries; no raw errors; enforce policy helpers.  
**First Implementation Priority:** Initialize repo with the selected starter command, add Prisma/Postgres + migrations, then wire auth/RBAC middleware.

## Architecture Completion Summary

### Workflow Completion

**Architecture Decision Workflow:** COMPLETED ✅  
**Total Steps Completed:** 8  
**Date Completed:** 2025-12-03T16:33:08+08:00  
**Document Location:** docs/architecture.md

### Final Architecture Deliverables

**📋 Complete Architecture Document**
- All architectural decisions documented
- Implementation patterns ensuring AI agent consistency
- Complete project structure with all files and directories
- Requirements to architecture mapping
- Validation confirming coherence and completeness

**🏗️ Implementation Ready Foundation**
- 0 architectural decisions enumerated here (see prior sections for detail)
- Implementation patterns defined
- Architectural components specified
- Requirements fully supported

**📚 AI Agent Implementation Guide**
- Technology stack choices
- Consistency rules to prevent conflicts
- Project structure with clear boundaries
- Integration patterns and communication standards

### Implementation Handoff

**For AI Agents:** This architecture document is your guide for implementing opencode. Follow all decisions, patterns, and structures exactly as documented.

**First Implementation Priority:** Initialize project with the documented starter command, add Prisma/Postgres + migrations, then wire auth/RBAC middleware.

**Development Sequence:**
1. Initialize project using documented starter template
2. Set up development environment per architecture
3. Implement core architectural foundations
4. Build features following established patterns
5. Maintain consistency with documented rules

### Quality Assurance Checklist

**✅ Architecture Coherence**
- [x] All decisions work together without conflicts
- [x] Technology choices are compatible
- [x] Patterns support the architectural decisions
- [x] Structure aligns with all choices

**✅ Requirements Coverage**
- [x] All functional requirements are supported
- [x] All non-functional requirements are addressed
- [x] Cross-cutting concerns are handled
- [x] Integration points are defined

**✅ Implementation Readiness**
- [x] Decisions are specific and actionable
- [x] Patterns prevent agent conflicts
- [x] Structure is complete and unambiguous
- [x] Examples are provided for clarity

### Project Success Factors

**🎯 Clear Decision Framework**  
Choices made collaboratively with clear rationale.

**🔧 Consistency Guarantee**  
Patterns and rules ensure multiple AI agents produce compatible code.

**📋 Complete Coverage**  
Requirements mapped to architecture and structure.

**🏗️ Solid Foundation**  
Starter and patterns provide a production-ready base.

---

**Architecture Status:** READY FOR IMPLEMENTATION ✅  
**Next Phase:** Begin implementation using the documented decisions and patterns.  
**Document Maintenance:** Update when major technical decisions change.
