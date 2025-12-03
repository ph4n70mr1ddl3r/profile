---
project_name: 'opencode'
user_name: 'Riddler'
date: '2025-12-03T16:42:31+08:00'
sections_completed: ['technology_stack']
existing_patterns_found: 0
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

- Architecture doc is the source of truth; stack captured there. No package.json present yet—pin versions when initialized: Next.js (current LTS), TypeScript strict, Prisma (Postgres), NextAuth, Redis client, Zod, TanStack Query, Playwright/Vitest to be added.
- Database: Postgres (managed Neon/Supabase), Prisma Migrate for schema.
- Hosting/deploy target: Vercel; managed Postgres/Redis; GitHub Actions CI.

## Critical Implementation Rules

- Always enforce tenant scoping: every DB query includes `tenant_id`; do not trust client-provided tenant—derive from session/context.
- Use standardized API envelopes: success `{ data, meta? }`; errors `{ error: { code, message, details? } }`; ISO 8601 dates only.
- Validate all inputs/outputs with Zod at API edges and forms; map camelCase API ↔ snake_case DB via mappers.
- Auth/RBAC: NextAuth sessions; role + tenant checks in middleware and service layer; policy helpers for “never show unverified as verified.”
- Error handling: never return raw errors; map typed errors to envelope codes; log with requestId/tenantId/userId/role; redact PII.
- Rate limiting: Redis-backed sliding window on auth, shortlist/report fetches, and webhooks; enforce per-tenant/user/IP as appropriate.
- Webhooks: require signature + nonce/TTL replay protection; reject unsigned/expired; log correlation ids.
- State/data: React Server Components first; client components for interactivity; TanStack Query for data/mutations; immutable updates.
- Naming: DB tables/columns snake_case; API fields camelCase; files kebab-case; components PascalCase; functions/vars camelCase.
- Tests: co-locate unit/integration; Playwright E2E in `tests/e2e` with role/tenant fixtures; cover verification, shortlist, degraded mode, and RBAC leakage.
