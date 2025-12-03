# Creator App Bootstrap

Baseline Next.js App Router scaffold with typed env validation, Prisma + Postgres, Redis helper with graceful fallback, Sentry telemetry, request ID middleware, and health endpoint.

## Stack Versions (checked 2025-12-03 via npmjs)
- next 16.0.6
- react 19.2.0
- prisma / @prisma/client 7.0.1
- next-auth 4.24.13
- @upstash/redis 1.35.7
- @sentry/nextjs 10.28.0

## Prereqs
- Node.js 20+
- Postgres database for `DATABASE_URL`
- Upstash Redis REST URL + token

## Setup
1. Install deps: `npm install`
2. Copy env: `cp .env.example .env.local` and fill values (DB, Upstash, Sentry, NextAuth, base URLs).
3. Run migrations: `npm run prisma:migrate:dev`
4. Start dev server: `npm run dev` (http://localhost:3000)

## Smoke Checks
- Health endpoint: `curl -i http://localhost:3000/api/health` (expect 200 with `{ data, meta.requestId }`; headers include `x-request-id`).
- Redis ping: `node -e "import('./dist?');"` (after build) or run the health endpoint; fallback is logged if not configured.
- Sentry test event: `node -e "const Sentry=require('@sentry/nextjs');Sentry.init({dsn:process.env.SENTRY_DSN});Sentry.captureMessage('telemetry-smoke');console.log('sent')"`
- Request ID: `curl -i http://localhost:3000/api/health -H 'x-request-id:test-123'` (response echoes the same ID).

## Notes
- Middleware sets/propagates `x-request-id`; tenant binding stub is present for extension in Story 1.2+.
- Structured logging via `src/lib/logger.ts`; logs include requestId/tenantId and ISO timestamps.
- Prisma schema lives in `prisma/schema.prisma` with snake_case mappings and indexes.
- Health endpoint envelope: `{ data, meta: { requestId } }`.
