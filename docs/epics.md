# opencode - Epic Breakdown

**Author:** Riddler  
**Date:** 2025-12-03T16:55:32+08:00  
**Project Level:** 0  
**Target Scale:** not specified

---

## Overview

This document provides the complete epic and story breakdown for opencode, decomposing the requirements from the PRD into implementable stories.

**Living Document Notice:** This is the initial version. It will be updated after UX Design and Architecture workflows add interaction and technical details to stories.

_Epics summary will be added during workflow execution._

---

## Epics Structure Plan

1. **Foundation, Auth, and Tenant Safety** — Delivers a secure, multi-tenant shell with RBAC, rate limiting, and audit scaffolding so every subsequent feature is isolated and trusted. Covers FR19–FR21, FR25, FR31–FR33 (guardrails), and underpins all flows.
2. **Verification and Trust Signals (Creators & Fans)** — Users earn badges/scores with recoverable flows and clear signals; prevents “unverified shown as verified.” Covers FR1–FR5, FR23.
3. **Briefs, Matching, and Intros** — Brands submit briefs; system generates verified/partial shortlists with signals and warnings; intros approved/declined cleanly. Covers FR6–FR10.
4. **Campaign Execution and Trust-Forward Reporting** — Runs campaigns from approved intros; reports with signals/flags, degraded/partial annotations, and exports. Covers FR11–FR18.
5. **Ops, Rules, and Degraded-Mode Control** — Ops queue for flags/overrides, rule changes, anomaly alerts, and explicit degraded-mode handling with audit. Covers FR24, FR29–FR33.
6. **Notifications, Webhooks, and Usage Plans** — Event notifications, signed webhooks with replay protection, and plan/usage enforcement. Covers FR26–FR28, FR34–FR36.

### Dependencies
- Epic 1 precedes all others (auth/tenant/rate limits/policies).
- Epic 2 depends on Epic 1 (auth/tenant) for secure verification state.
- Epic 3 depends on Epics 1–2 (need verified profiles to shortlist).
- Epic 4 depends on Epic 3 (campaigns stem from intros) and Epic 2 (signals in reports).
- Epic 5 overlays Epics 2–4 (ops/rules/degraded control).
- Epic 6 integrates with Epics 2–5 (events, webhooks, usage gating).

---

## Technical Context for Epics

- Stack: Next.js App Router (TS), Prisma/Postgres (tenant_id on all domain tables), NextAuth for auth, Redis for rate limits/cache, Zod at edges, Sentry/logging, GitHub Actions CI.
- API envelope: `{ data, meta? }` success; `{ error: { code, message, details? } }` failures; ISO 8601 dates; request IDs.
- Security/RBAC: middleware enforces session + tenant; policy helpers for “never show unverified as verified”; role checks at route + service layers.
- Trust UX: badges/signals/warnings inline; partial/degraded annotations; accessible chips/tooltips; WCAG AA.
- Performance/Resilience: p95 <2s core flows; shortlist SLA <24h; degraded mode when providers fail; audit trail for signals/overrides/report annotations.

---

## Context Validation

- PRD loaded: docs/prd.md (functional and non-functional requirements present).
- Architecture loaded: docs/architecture.md (technical decisions, stack, and patterns present).
- UX Design loaded: docs/ux-design-specification.md (journeys, trust UX patterns, and accessibility guidance present).
- Prerequisite check: all required documents available; workflow can proceed.

---

## Functional Requirements Inventory

- FR1: Creators can complete a verification flow to generate an authenticity score and badge.
- FR2: Fans can complete a lightweight verification flow to earn a trusted badge.
- FR3: The system can surface verification signals (e.g., authenticity indicators, fraud/deepfake flags) on creator profiles and campaign assets.
- FR4: Users can recover from failed verification with guided remediation steps.
- FR5: The system can prevent unverified profiles from being displayed as verified in any context.
- FR6: Brand users can create and submit campaign briefs with target audience/vertical and objectives.
- FR7: The system can generate a verified shortlist for a brief, including authenticity signals for each match.
- FR8: The system can return a partial shortlist with warnings when sufficient qualified matches are unavailable.
- FR9: Brand users can request and approve intros with shortlisted creators.
- FR10: Creators can accept/decline intro requests and manage availability.
- FR11: Brand users can initiate a verified campaign from an approved intro.
- FR12: Creators can confirm participation details for a campaign.
- FR13: The system can track campaign status and key milestones (e.g., approved, in-progress, completed).
- FR14: Brand users and creators can exchange campaign updates and assets within the platform.
- FR15: Brand users can view campaign reports that include authenticity scores, fraud/deepfake flags, and engagement quality signals.
- FR16: Brand users can export campaign reports and underlying signals (e.g., CSV/JSON).
- FR17: The system can display diversification/risk cues (e.g., channel risk guidance) within reports or briefs.
- FR18: The system can show warnings when operating in degraded mode (e.g., partial signals) and annotate reports accordingly.
- FR19: Organizations can operate in a shared multi-tenant environment with tenant-scoped data isolation.
- FR20: Brand Admins can manage billing, team membership, and permissions.
- FR21: Brand Members can create/edit briefs and review shortlists per assigned permissions.
- FR22: Creators can manage their profile, verification status, and collaboration settings.
- FR23: Fans can manage their trusted badge status and related profile indicators.
- FR24: Ops/Admin users can review flags, apply overrides, and update rules.
- FR25: The system can enforce role-based access across briefs, intros, reports, and admin actions.
- FR26: Users can receive notifications for key events (e.g., shortlist ready, intro requests, verification outcomes, alerts).
- FR27: The system can deliver webhook callbacks for key events (e.g., shortlist generated, campaign status changes).
- FR28: Users can export or download data relevant to their role (e.g., reports, shortlists).
- FR29: Ops/Admin can review flagged verification cases and approve/deny with auditability.
- FR30: Ops/Admin can adjust verification rules and thresholds.
- FR31: The system can enter degraded mode when signal providers are unavailable and maintain guardrails.
- FR32: The system can maintain an audit trail for verification signals, overrides, and report annotations.
- FR33: The system can surface alerts for anomalies (e.g., suspected fraud or signal gaps) to Ops/Admin.
- FR34: The system can enforce usage limits (e.g., briefs, shortlists, campaigns) by plan/tier.
- FR35: Brand Admins can view current plan, usage, and overage status.
- FR36: The system can gate advanced features (e.g., deeper signals, SLA-backed reporting) by plan/tier.

---

## FR Coverage Map

- Epic 1: FR19–FR21, FR25, FR31–FR33 (foundation guards all flows).
- Epic 2: FR1–FR5, FR23 (verification, badges, signals, recovery).
- Epic 3: FR6–FR10 (briefs, verified/partial shortlist, intros).
- Epic 4: FR11–FR18 (campaigns, reports, exports, diversification/risk cues, degraded annotations).
- Epic 5: FR24, FR29–FR33 (ops queue, overrides, rules, alerts, degraded mode).
- Epic 6: FR26–FR28, FR34–FR36 (notifications, webhooks, usage/plan gating).

---

<!-- Epic sections will be appended during workflow execution -->

## Epic 1: Foundation, Auth, and Tenant Safety

Deliver a secure multi-tenant shell with RBAC, rate limiting, and audit scaffolding so all subsequent features inherit correct isolation and trust controls.

### Story 1.1: Project Bootstrap, Env, and Telemetry
As an engineer, I want the baseline Next.js + Prisma + Redis + Sentry wiring so the app runs with typed env validation and telemetry.

**Acceptance Criteria**
- `next.config.js`, `tsconfig`, `eslint` baseline created per architecture patterns; `src/lib/config/env.ts` validates required envs (DB URL, Redis URL, Sentry DSN, NextAuth secrets).
- Prisma schema exists with base `tenant` and `user` tables (snake_case, tenant_id where applicable) and migrations run locally.
- Sentry (or equivalent) initialized on both server and client with request ID correlation.
- Health route responds 200 with build info; logs structured JSON with requestId and tenantId when present.

**Technical Notes**
- Use `@/*` alias; ensure `tenant_id` included on domain tables from the start.
- Enforce ISO date handling; add request ID middleware.
- Redis client ready for rate limiting (no-op fallback if env missing in dev).

### Story 1.2: Auth Session with Tenant Binding
As an authenticated user, I want sessions bound to a tenant so all data access is scoped automatically.

**Acceptance Criteria**
- NextAuth configured with Postgres adapter; session includes `tenantId`, `role`, `userId`.
- Middleware enforces authenticated access for `(app)` routes and injects `tenantId` into request context; redirects unauthenticated users.
- Tenant scoping helper ensures all repository queries include `tenant_id` and rejects cross-tenant access.

**Technical Notes**
- Add policy helper: `requireTenant(ctx)` and `assertRole(ctx, roles)`.
- Rate limit auth endpoints via Redis sliding window (per IP/user).

### Story 1.3: RBAC and Route Guards
As a tenant admin/user, I want role-aware access so only allowed actions are permitted.

**Acceptance Criteria**
- Roles: Brand Admin, Brand Member, Creator, Fan, Ops/Admin stored with tenant binding.
- Route guards at API and page level enforce role permissions for briefs, shortlists, reports, verification, ops.
- Unauthorized requests return 403 with envelope `{ error: { code: 'forbidden', message } }`; events logged with requestId + tenantId + userId.

**Technical Notes**
- Central RBAC map in `src/lib/rbac.ts`; reuse in middleware and services.

### Story 1.4: Rate Limiting and Audit Scaffolding
As a platform owner, I want rate limits and audit scaffolding to protect trust surfaces.

**Acceptance Criteria**
- Redis-backed rate limits on auth, verification submission, shortlist/report fetch, and webhooks; returns 429 with retry headers.
- Audit trail table created (append-only) for verification signals, overrides, report annotations; repository helper to append audit entries with actor and tenant.
- Error envelope never leaks raw errors; logs include sanitized details.

**Technical Notes**
- Use consistent event names (e.g., `verification.submitted`, `shortlist.requested`, `report.viewed`).

---

## Epic 2: Verification and Trust Signals (Creators & Fans)

Trust-first verification flows with badges/scores, recovery, and guardrails that prevent unverified entities from appearing verified.

### Story 2.1: Creator Verification Flow with Signals
As a creator, I want to complete verification and receive an authenticity score/badge with clear status and recovery.

**Acceptance Criteria**
- API: `POST /api/verification/creator` validates with Zod; persists verification session with tenant_id and status (`pending/partial/failed/verified`).
- Signal ingestion adapter stubbed with provider metadata; status endpoint returns signals, flags, timestamps.
- UI flow shows progress, partial, and failed states; verified badge set only when signals present and status=verified.
- Recovery guidance shown on partial/failed; audit entry added for each status change with actor.

**Technical Notes**
- Use background job/poll/push pattern; p95 interaction steps <2s, long work async with status poll.
- Store signal artifacts (provider, score, flags) with provenance for audit.

### Story 2.2: Fan Badge Flow
As a fan, I want a lightweight verification to earn a badge for interactions.

**Acceptance Criteria**
- API: `POST /api/verification/fan` with minimal fields; status endpoint mirrors creator shape.
- UI completes in <=5 minutes; badge shows on fan interactions; partial/failed states with remediation.
- Unverified fans never rendered as verified; recovery allows retry.

**Technical Notes**
- Reuse validation and degraded-mode annotations; tenant-scoped queries; rate limit submissions.

### Story 2.3: Trust Signals Rendering and Guardrails
As any user, I want transparent signals and warnings so I can trust verification states.

**Acceptance Criteria**
- Profiles show authenticity score, signals/flags, last-updated, and degraded/partial annotations inline with badges.
- Missing/partial signals render warning banners; verified badge never shown without signal payload.
- Tooltips/expanders list signal provenance and timestamps.

**Technical Notes**
- Semantic tokens for verified/success, warning/partial, error; ensure WCAG AA contrast/focus.

### Story 2.4: Degraded Mode and Recovery
As the platform, I want safe degraded behavior when signal providers fail.

**Acceptance Criteria**
- Central degraded-mode flag gates verification completion; surfaces warnings in UI + API responses.
- Status changes recorded in audit trail; remediation guidance shown inline.
- Ops visibility: expose degraded state via API for later ops console use.

**Technical Notes**
- Cache invalidation for degraded flag; log toggles with requestId/tenantId/userId.

**Epic 2 Complete**
- FR Coverage: FR1–FR5, FR23.
- Technical Context Used: NextAuth sessions, Prisma tenant scoping, Redis rate limits, signal provider adapters (stub), audit trail.
- UX Patterns: badges/chips, warning banners, tooltips, partial/degraded annotations.

---

## Epic 3: Briefs, Matching, and Intros

Brands submit briefs; the system produces verified/partial shortlists with signals and warnings; intros are managed cleanly.

### Story 3.1: Brief Submission and Intake
As a brand user, I want to submit a brief with targets and objectives to start matching.

**Acceptance Criteria**
- API: `POST /api/briefs` (Zod-validated) creates brief with tenant_id, owner, objectives, target audience/vertical; stores status.
- UI form with inline validation; success returns brief summary; unauthorized roles blocked.
- Audit entry on creation; rate limited.

**Technical Notes**
- Use camelCase API, snake_case DB; envelope responses; requestId logging.

### Story 3.2: Shortlist Generation with Verified/Partial Output
As a brand user, I want a verified shortlist (or partial with warnings) tied to my brief.

**Acceptance Criteria**
- Service matches against verified creators (Epic 2 output) and returns ≥5 if available; if not, returns partial shortlist with warnings and rationale.
- API: `POST /api/shortlists` triggers generation; status endpoint reports progress and partial/degraded flags.
- Shortlist items include signals (scores/flags) and provenance; partial/warning annotations required when below target or degraded.

**Technical Notes**
- Background worker allowed; store match metadata; ensure tenant scoping on queries; log warnings for partial/degraded states.

### Story 3.3: Intro Requests and Approvals
As a brand user/creator, I want to request and respond to intros for shortlisted creators.

**Acceptance Criteria**
- API: `POST /api/shortlists/{id}/intros` creates intro request; creator can accept/decline via `PATCH /api/intros/{id}`.
- Status transitions audited; notifications stubbed (Epic 6 to deliver).
- UI shows intro status; only shortlisted creators can be targeted; role/RBAC enforced.

**Technical Notes**
- Consistent status enums; prevent forward dependencies (intros only after shortlist ready).

### Story 3.4: Shortlist UI with Trust Annotations
As a brand user, I want to view shortlists with clear trust signals and warnings.

**Acceptance Criteria**
- UI renders cards/table with badges, signals, warnings for partial/degraded states; includes “why this match” notes.
- Sorting/filtering respects tenant and roles; performance meets p95 <2s with server data fetching.
- Empty/partial states clearly annotated; never hide warnings.

**Technical Notes**
- Use server components for data fetch; client components for interactions; accessibility for chips/tooltips.

**Epic 3 Complete**
- FR Coverage: FR6–FR10.
- Technical Context Used: Matching service on Prisma, tenant/RBAC guards, Redis rate limits, audit trails.
- UX Patterns: cards/tables with badges, warning annotations, empty/partial states.

---

## Epic 4: Campaign Execution and Trust-Forward Reporting

Runs campaigns from approved intros with transparent signals, exports, and degraded/partial annotations.

### Story 4.1: Campaign Initiation and Status Tracking
As a brand user, I want to start a campaign from an approved intro and track its status.

**Acceptance Criteria**
- API: `POST /api/campaigns` from approved intro; statuses (`draft/approved/in-progress/completed`) with timestamps and tenant scoping.
- Creators confirm participation via `PATCH /api/campaigns/{id}`; status changes audited.
- UI shows status timeline with trust badges (if verification tied) and warnings when degraded.

**Technical Notes**
- Enforce prerequisite: intro must be accepted; role checks for brand vs creator actions.

### Story 4.2: Reporting with Signals, Flags, and Exports
As a brand user, I want reports that include authenticity signals, fraud/deepfake flags, and exports.

**Acceptance Criteria**
- API: `GET /api/reports/{id}` returns signals, flags, audit annotations, and export links (CSV/JSON).
- Reports show authenticity scores, fraud flags, engagement quality; warnings when data partial/degraded.
- Export endpoints enforce role + tenant; log download events with requestId/tenantId/userId.

**Technical Notes**
- Use consistent envelope; generate signed download links or streamed CSV/JSON.

### Story 4.3: Diversification/Risk Cues and Degraded Annotations
As a brand user, I want risk guidance and clear degraded-mode annotations in reports.

**Acceptance Criteria**
- Reports include diversification/risk cues (e.g., channel risk guidance) based on PRD; partial signals flagged.
- Degraded mode clearly shown when providers fail; claims limited; annotations persisted in audit trail.
- UI tooltips/badges explain risk cues; adherence to WCAG AA.

**Technical Notes**
- Reuse trust tokens; ensure consistent warning semantics across shortlist/report views.

**Epic 4 Complete**
- FR Coverage: FR11–FR18.
- Technical Context Used: Campaign/report services, audit trail, exports, degraded-mode flag.
- UX Patterns: timelines, badges, warning/annotation banners, exports CTA with explanations.

---

## Epic 5: Ops, Rules, and Degraded-Mode Control

Ops/Admin manage flags, overrides, rules, and degraded mode with full auditability.

### Story 5.1: Ops Flag Queue with Approve/Deny
As an Ops/Admin user, I want to review flagged verification cases and take action with audit.

**Acceptance Criteria**
- API: `GET /api/ops/flags` and `PATCH /api/ops/flags/{id}` to approve/deny with reason; actions audited.
- UI queue shows signals snapshot, reason, and actions; only Ops/Admin roles allowed.
- Decisions propagate to verification status and badges; no cross-tenant leakage.

**Technical Notes**
- Ensure idempotent actions; log actor and timestamp; rate limit ops actions moderately.

### Story 5.2: Rule Management and Overrides
As an Ops/Admin user, I want to adjust verification rules/thresholds safely.

**Acceptance Criteria**
- API: `PATCH /api/ops/rules` to update thresholds/config; changes logged to audit with old/new values.
- Validation prevents unsafe ranges; requires Ops/Admin role and CSRF/session guard.
- UI shows current rules and history with actor + timestamp.

**Technical Notes**
- Store rules versioned; ensure rollback path; consider feature flag-style toggles.

### Story 5.3: Degraded Mode Activation and Alerts
As an Ops/Admin user, I want to activate/deactivate degraded mode and see anomaly alerts.

**Acceptance Criteria**
- API: `POST /api/ops/degraded` toggles mode with reason and TTL; audit entry recorded.
- Alerts surface anomalies (suspected fraud/signal gaps) with link to flags queue.
- UI shows active degraded status globally; all verification/reporting surfaces respect the flag.

**Technical Notes**
- Cache/propagate flag across app; ensure consistent warning state; log to telemetry.

**Epic 5 Complete**
- FR Coverage: FR24, FR29–FR33.
- Technical Context Used: Audit trail, Ops APIs, degraded-mode flag, alert surfacing.
- UX Patterns: admin tables, banners for degraded mode, action modals with confirmations.

---

## Epic 6: Notifications, Webhooks, and Usage Plans

Notifications for key events, signed webhooks with replay protection, and plan/usage enforcement.

### Story 6.1: Notification Dispatch for Key Events
As a user, I want notifications for shortlist readiness, intros, verification outcomes, and alerts.

**Acceptance Criteria**
- Notification service triggers on events: shortlist ready, intro request/response, verification success/failure, degraded mode alert.
- Delivery channels stubbed (e.g., email/webhook placeholder); enqueue/send logic respects rate limits and tenant scoping.
- Users can view recent notifications in-app; unread/read state stored.

**Technical Notes**
- Event names match earlier patterns; queue-friendly design; log requestId/tenantId.

### Story 6.2: Signed Webhooks with Replay Protection
As an integrator, I want signed webhooks for key events with replay protection.

**Acceptance Criteria**
- Webhook endpoints sign payloads with shared secret; include nonce + timestamp; receivers can verify.
- Replay protection via nonce stored in Redis with TTL; repeated nonce rejected with 409/appropriate error envelope.
- Docs example provided; verification failures logged with requestId.

**Technical Notes**
- Use HMAC with configured secret; consistent headers (`X-Signature`, `X-Nonce`, `X-Timestamp`).

### Story 6.3: Usage Limits and Plan Enforcement
As a platform owner, I want plan/usage enforcement on briefs, shortlists, and campaigns.

**Acceptance Criteria**
- Usage counters per tenant for briefs/shortlists/campaigns; plan tiers define caps and feature gates.
- API and UI block or warn when limits reached; responses use standard envelope with actionable message.
- Plan/usage view available to Brand Admin; overage status displayed.

**Technical Notes**
- Store usage in Postgres with periodic cache in Redis; ensure atomic increments; audit plan changes.

**Epic 6 Complete**
- FR Coverage: FR26–FR28, FR34–FR36.
- Technical Context Used: Event system, Redis replay protection, plan/usage models, notification store.
- UX Patterns: notifications list, banners for limits, docs snippet for webhooks.

---

## FR Coverage Matrix

| FR | Epic/Story Coverage |
| --- | --- |
| FR1 | Epic 2.1 |
| FR2 | Epic 2.2 |
| FR3 | Epic 2.3 |
| FR4 | Epic 2.1, 2.4 |
| FR5 | Epic 2.1, 2.3, 2.4 |
| FR6 | Epic 3.1 |
| FR7 | Epic 3.2 |
| FR8 | Epic 3.2 |
| FR9 | Epic 3.3 |
| FR10 | Epic 3.3 |
| FR11 | Epic 4.1 |
| FR12 | Epic 4.1 |
| FR13 | Epic 4.1 |
| FR14 | Epic 4.1, 4.2 |
| FR15 | Epic 4.2 |
| FR16 | Epic 4.2 |
| FR17 | Epic 4.3 |
| FR18 | Epic 4.3 |
| FR19 | Epic 1.2 |
| FR20 | Epic 1.3 |
| FR21 | Epic 1.3 |
| FR22 | Epic 2.1 |
| FR23 | Epic 2.2 |
| FR24 | Epic 5.1 |
| FR25 | Epic 1.3 |
| FR26 | Epic 6.1 |
| FR27 | Epic 6.2 |
| FR28 | Epic 4.2 |
| FR29 | Epic 5.1 |
| FR30 | Epic 5.2 |
| FR31 | Epic 1.4, 2.4 |
| FR32 | Epic 1.4, 5.1, 5.2 |
| FR33 | Epic 5.3 |
| FR34 | Epic 6.3 |
| FR35 | Epic 6.3 |
| FR36 | Epic 6.3 |

---

## Summary

- Epics defined (6) with story-level acceptance criteria and technical notes aligned to architecture (Next.js, Prisma/Postgres, NextAuth, Redis, Zod, Sentry).
- FR coverage complete (FR1–FR36 mapped) with trust-first constraints (tenant scoping, audit, degraded-mode annotations).
- Trust UX patterns embedded (badges/signals/warnings, tooltips, partial/degraded states) and performance targets called out (p95 <2s, shortlist SLA <24h).
- Ops/degraded mode and usage/plan enforcement included to keep guardrails intact across verification, shortlists, and reports.
- Notifications/webhooks and exports included with signature/replay protection and standard envelopes.

---

_For implementation: Use the `create-story` workflow to generate individual story implementation plans from this epic breakdown._

_This document will be updated after UX Design and Architecture workflows to incorporate interaction details and technical decisions._
