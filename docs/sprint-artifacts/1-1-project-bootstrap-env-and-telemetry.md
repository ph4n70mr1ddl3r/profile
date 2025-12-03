# Story 1.1: project-bootstrap-env-and-telemetry

Status: ready-for-dev

## Story

As an engineer, I want the baseline Next.js + Prisma + Redis + Sentry wiring so the app runs with typed env validation and telemetry.

## Acceptance Criteria

1. Next.js App Router project with TypeScript, eslint, and `@/*` alias configured (matches architecture patterns); `tsconfig.json` and `next.config.js` present; package.json pins baseline versions (e.g., next@14.x, react@18.x, prisma@5.x, next-auth@4.x or auth.js@stable, @upstash/redis@1.x, @sentry/nextjs@7.x). Record a latest-stable check (date + source + version list) before install and block if missing. Latest check (2025-12-03, source: npmjs): next 16.0.6; react 19.2.0; prisma 7.0.1; @prisma/client 7.0.1; next-auth 4.24.13; @upstash/redis 1.35.7; @sentry/nextjs 10.28.0.
2. `src/lib/config/env.ts` validates required env vars (DB URL, Redis URL, Sentry DSN, NextAuth secrets, base app URL) with Zod; rejects startup on missing/invalid config; `.env.example` documents required keys; include local runbook to load envs safely (no secrets checked in).
3. Prisma schema defines base `tenant` and `user` tables (snake_case, `tenant_id` included where applicable) with explicit fields: tenant (id uuid pk, name, slug unique, created_at, updated_at); user (id uuid pk, tenant_id fk, email unique, role enum, created_at, updated_at). Index on `(tenant_id, email)`; migrations run locally and generate client.
4. Sentry (or equivalent) initialized for server and client paths with request ID correlation and PII scrubbing; structured JSON logging includes `requestId` and `tenantId` when present; errors sanitized before logging.
5. Health route responds 200 with build info and env readiness; envelope uses `{ data, meta? }` and includes requestId. Log includes `{ level, message, requestId, tenantId? }`.
6. Redis client ready for rate limiting with safe no-op fallback in dev if envs missing; rate-limit defaults are safe and logged when fallback is active.
7. ISO date handling enforced; request ID middleware added so downstream handlers/logs include a consistent correlation ID; session middleware enforces tenant binding.
8. Performance/NFR: p95 latency <2s for health route; warm API paths p95 <500ms with logging overhead <5ms; cold-start budget documented; structured logging does not block response; DB queries use indexes and avoid full scans.
9. Deployment/env runbook includes: copy `.env.example` to `.env.local` and fill secrets; run `npm run prisma:migrate:dev` (or equivalent); run telemetry smoke (Sentry test event) and Redis ping/fallback log; capture outcomes in notes.
10. Cross-story dependencies mapped to Epic 1: Story 1.2 (auth session + tenant binding), 1.3 (RBAC/route guards), 1.4 (rate limiting/audit); clarify reuse paths to extend env/middleware/logging instead of duplicating.
11. Accessibility/WCAG baseline in place: trust-state tokens/colors in Tailwind config, linting for a11y enabled, and UI components follow UX accessibility guidance.
12. Scope boundaries: bootstrap only (no feature UIs); telemetry limited to baseline Sentry/logging; auth wiring stops at scaffolding (full flows in Story 1.2+); rate limiting scaffolding only (full policies later).

## Tasks / Subtasks

- [ ] Initialize Next.js App Router project (TS, Tailwind, eslint) with `@/*` alias.
- [ ] Add env schema at `src/lib/config/env.ts` using Zod; surface typed config import across codebase; `.env.example` captured and checked in (no secrets).
- [ ] Configure Prisma (`prisma/schema.prisma`) with `tenant` and `user` tables; include fields/indexes per acceptance criteria; run `prisma migrate dev` locally and check in migrations.
- [ ] Wire NextAuth scaffolding (adapter ready) and session typing that carries `tenantId`, `role`, `userId`; middleware enforces tenant binding.
- [ ] Add Redis client helper with graceful fallback for missing config in dev; log when fallback is active; set safe defaults for rate limiting.
- [ ] Instrument Sentry on server/client; attach requestId/tenantId to logs/events; enable PII scrubbing; sanitize errors before logging.
- [ ] Add request ID middleware; emit structured logs with `{ level, message, requestId, tenantId? }`; ensure ISO timestamps.
- [ ] Implement `app/api/health/route.ts` returning 200 and build info; include envelope and requestId; ensure p95<2s target and cold-start budget noted; add warm-path perf targets in README notes.
- [ ] Ensure `next.config.js`, `tsconfig.json`, `.eslintrc`, `tailwind.config.js` align with architecture decisions.
- [ ] Pin baseline versions in package.json (next, react/react-dom, prisma/@prisma/client, next-auth or auth.js, @upstash/redis, @sentry/nextjs); record latest-stable check (date + source + version list) before install; note outcomes in README/story.
- [ ] Apply accessibility tokens and linting per UX doc (Tailwind theme tokens for trust states, enable eslint a11y rules, ensure base components honor WCAG AA).
- [ ] Add deployment/env runbook for local/dev: env load, migrations, telemetry smoke (Sentry test event, Redis ping or fallback log) with exact commands and expected results.
- [ ] Document cross-story dependencies (1.2 auth session, 1.3 RBAC/route guards, 1.4 rate limiting/audit) and reuse guidance (extend env/middleware/logging utilities, do not duplicate).
- [ ] Add debugging playbook: env validation command, Sentry test command, Redis connectivity/fallback check, how to view structured logs, and where to record results.

## Developer Context

- Target outcome: bootstrapped app that enforces typed env config, tenant-aware scaffolding, and telemetry from day 0. This unblocks all downstream stories that rely on consistent env, auth, and observability.
- Dependencies: architecture stack (Next.js App Router + TypeScript + Tailwind), Prisma/Postgres, NextAuth, Redis, Sentry, Zod for validation, request ID middleware.
- Guardrails: never serve without valid envs; tenant scoping baked into schema/middleware; all APIs use the standard envelope and include correlation IDs.

## Technical Requirements

- App scaffold: Next.js (App Router), TypeScript, Tailwind; `@/*` import alias; linting via eslint.
- Config: Zod schema in `src/lib/config/env.ts`; typed export used by server/client initializers; fail fast on invalid env.
- DB: Postgres via Prisma; base tables `tenants` and `users` (snake_case); include `tenant_id` on domain tables from the start; migrations tracked in repo.
- Auth: NextAuth wired to Postgres adapter; session shape includes `tenantId`, `role`, `userId`; middleware enforces auth + tenant.
- Logging/telemetry: Sentry on server/client; structured logs (JSON) with requestId + tenantId; ISO 8601 timestamps; p95 targets in logs.
- Rate limiting: Redis helper with sliding window available for auth and later APIs; no-op fallback when Redis config absent in dev.
- API envelope: `{ data, meta? }` success; `{ error: { code, message, details? } }` failures; include `X-Request-Id` header.

## Architecture Compliance

- Follow file layout from docs/architecture.md: feature-first under `src/app/(app)/...` and `src/app/api/.../route.ts`.
- Enforce tenant scoping in DB schema and helpers; policy helpers live in `src/lib/rbac.ts`.
- Use Zod for all input validation; align naming: snake_case DB, camelCase API.
- Instrument Sentry/logging per architecture; avoid raw errors and stick to envelope with ISO dates.

## Library / Framework Requirements

- Next.js App Router (TypeScript), Tailwind.
- Prisma with Postgres adapter; `@prisma/client` generated after migrations.
- NextAuth with Postgres adapter; session typing extended for tenant/role.
- Redis client (Upstash or compatible) with graceful fallback.
- Zod for env and request validation.
- Sentry (or equivalent) for server/client telemetry with requestId/tenantId context.

## File Structure Requirements

- `src/lib/config/env.ts` — Zod env schema and typed config export.
- `prisma/schema.prisma` — `tenant` and `user` tables; migrations under `prisma/migrations`.
- `src/middleware.ts` — auth + tenant + requestId middleware; add rate-limit hook when ready.
- `src/app/api/health/route.ts` — health endpoint returning envelope with build info, requestId.
- `src/lib/rbac.ts` — policy helpers (stubs acceptable for bootstrap).
- `src/lib/telemetry.ts` or Sentry init files — server/client initialization with scope for IDs.

## Testing Requirements

- Unit: env schema rejects missing/invalid vars; Redis helper returns no-op when config absent; health route returns 200 and envelope shape.
- Integration (once services exist): Prisma connects with tenant/user schema; NextAuth session includes tenantId/role; requestId middleware sets header and log field.
- Tooling: lint/typecheck pass (`eslint`, `tsc`); consider basic Playwright smoke for health route once app scaffolded.

## Dev Notes

- Enforce ISO date handling and consistent requestId propagation from middleware to logs/Sentry.
- Keep safety net: default Redis client stub in dev to avoid crashes; logs should note when fallback is active.
- Set up `.env.example` to document required vars (DB, REDIS, SENTRY_DSN, NEXTAUTH_SECRET, NEXTAUTH_URL).
- Use structured logging utilities to avoid ad-hoc console output.

## Project Structure Notes

- Follow architecture directory guidance (feature-first under `src/app/(app)/...`; API routes in `src/app/api/...`).
- Keep shared libs in `src/lib` (auth, db, validation, telemetry, rbac); services/repositories under `src/server` when added.
- Use app route groups for auth-protected areas and to keep tenant scoping centralized.

## References

- Story source: docs/epics.md (Epic 1, Story 1.1).
- Architecture patterns: docs/architecture.md.
- Product requirements and NFRs: docs/prd.md (perf p95<2s, guardrails on verification, audit/logging).
- UX principles (trust cues, clarity, accessibility): docs/ux-design-specification.md.

## Dev Agent Record

### Context Reference

- PRD: docs/prd.md
- Architecture: docs/architecture.md
- Epics: docs/epics.md
- UX: docs/ux-design-specification.md

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

- Request ID propagation and logging configured per middleware.

### Completion Notes List

- Story drafted with status `ready-for-dev`; sprint-status updated.
- No previous-story learnings (first story of epic).
- No git analysis performed (not applicable for first story).

### File List

- docs/sprint-artifacts/1-1-project-bootstrap-env-and-telemetry.md
- docs/sprint-artifacts/sprint-status.yaml (updated status)
