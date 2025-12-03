---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
inputDocuments:
  - docs/analysis/research/market-creator-brand-fan-platform-research-2025-12-03.md
  - docs/analysis/brainstorming-session-2025-12-03.md
workflowType: 'prd'
lastStep: 10
project_name: 'opencode'
user_name: 'Riddler'
date: '2025-12-03'
---

# Product Requirements Document - opencode

**Author:** Riddler  
**Date:** 2025-12-03

## Executive Summary

Building a verification-first creator–brand–fan platform that proves audience authenticity, bakes in fraud/deepfake resistance, and gives brands transparent quality/reporting. Starting focus: micro/nano creators and LinkedIn/B2B segments, with diversification playbooks to reduce single-platform risk.

### What Makes This Special

- Audience-quality scoring with transparent signals (anti-bot/deepfake) as the core product.
- Fraud/verification woven into collab workflows—not an add-on—plus clear reporting for brands.
- Diversification guidance that de-risks TikTok-style shocks; verified status unlocks better matches and faster approvals.

## Project Classification

**Technical Type:** saas_b2b (web app)  
**Domain:** creator/influencer marketing (general)  
**Complexity:** medium (trust/data-integrity caveats; not regulated)

Classification notes: “platform, dashboard, brands, creators” → saas_b2b; web delivery implied. Trust/fraud focus adds data-integrity risk but no regulated domain signals.

## Success Criteria

### User Success
- Creators: Complete profile + verification and publish a verifiable audience-quality score within 15 minutes; ≥70% of verified creators get a brand-ready profile score (or higher) within 24 hours.
- Brands: Receive a verified shortlist (≥5 matches) with authenticity scores in <24 hours of posting a brief; ≥50% of briefs convert to at least one vetted intro.
- Fans/Followers: Earn a trusted badge with <5 minutes of friction; badge adoption on ≥30% of engaged fans for targeted creators within first 30 days of those creators joining.

### Business Success
- 3 months: 200 verified creators, 50 active brands, 50 verified campaigns completed; ≥60% campaign satisfaction (brand-side); 30% creator week-4 retention.
- 12 months: 2,000 verified creators, 300 active brands, 500 verified campaigns completed; ≥75% campaign satisfaction; 45% creator month-3 retention; 20% MoM growth in verified campaigns.
- One headline KPI: Verified campaigns completed per month, with ≥75% brand satisfaction.

### Technical Success
- Authenticity/quality: Reduce suspected invalid/low-quality engagement by ≥50% vs baseline for participating campaigns; surface transparent signals (anti-bot/deepfake) in every creator profile and campaign report.
- Reliability/perf: Core flows (verify creator/fan, generate shortlist, issue reports) p95 < 2s; uptime 99.9% for verification and shortlist services.
- Data integrity: No unverified profiles shown as “verified”; audit trail on all verification signals in reports.

### Measurable Outcomes
- Time-to-value: Creator onboarding + verification < 15 minutes; brand shortlist delivery < 24 hours.
- Conversion: Brief-to-intro ≥50%; intro-to-campaign ≥30%.
- Adoption: Creator verification completion ≥70%; fan badge adoption ≥30% for target cohorts.
- Quality: ≥50% reduction in invalid engagement indicators on participating campaigns.

## Product Scope

### MVP - Minimum Viable Product
- Verification pipeline for creators/fans with transparency (signals visible in profiles/reports).
- Brand brief intake → automated verified shortlist generation; basic reporting with authenticity score and fraud/deepfake flags.
- Micro/nano and LinkedIn/B2B segments as initial focus; simple diversification guidance in reporting.
- Audit trail + correctness guardrails (no unverified profiles shown as verified).

### Growth Features (Post-MVP)
- Deeper diversification playbooks and channel-risk insights; automated alerts on fraud/risk spikes.
- SLA-backed reporting, richer collaboration workflows, and faster approvals for verified matches.
- Fan/creator perks tied to verified status; reputation tiers.

### Vision (Future)
- Standardized authenticity score adopted across creator/brand ecosystems; continuous fraud/deepfake detection with third-party audits.
- Cross-platform orchestration with automated failover guidance when platform risk changes.

## Innovation & Novel Patterns

### Detected Innovation Areas
- Trust-first scoring as a product surface: audience-quality signals + anti-bot/deepfake flags embedded in profiles, shortlists, and reports.
- Workflow automation with AI assist: brief intake → verified shortlist with signals and partial-list warnings; guided recovery for failed verifications.
- Risk-aware orchestration: diversification cues, anomaly alerts, and degraded-mode operations when signal providers hiccup.

### Market Context & Competitive Landscape
- Existing influencer platforms emphasize reach and basic fraud checks; few lead with transparent authenticity scoring plus partial/warning flows and diversified channel risk guidance.
- Verification-as-core differentiator positions us beyond “marketplace” into “proof + matching.”

### Validation Approach
- A/B: shortlist with signals vs. without; measure lift in brand shortlist acceptance and fraud/invalid engagement reduction.
- Time-to-value: creator verification <15 min; brand shortlist <24h; fan badge <5 min—track completion and drop-off.
- Accuracy/backtests: compare authenticity scores against known-bad/known-good cohorts; monitor false positives/negatives in flagged cases.

### Risk Mitigation
- Fallbacks: partial shortlist with warnings; never show unverified as verified; gentle recovery flows for failed badge/creator verification.
- Degraded mode: if signal providers fail, surface warnings and limit claims; keep audit trails intact.
- Human-in-loop: ops queue for edge cases; rule updates when anomalies spike.

## SaaS B2B Specific Requirements

### Project-Type Overview
- Multi-tenant by default (shared), with logical isolation; consider hybrid/single-tenant only for high-risk brands later.
- Roles/permissions centered on brand teams, creators, fans, and ops; fine-grained approvals for briefs, intros, and reports.
- Usage-based tiers to align with verified campaign volume and reporting depth; verification as a core value lever.

### Technical Architecture Considerations
- Tenant model: shared multi-tenant; per-tenant data isolation; scoped queries by tenant; audit trails per tenant.
- RBAC: roles for Brand Admin, Brand Member, Creator, Fan, Ops/Admin; approvals for shortlist/intros; no unverified shown as verified.
- Subscription/tiers: free trial with limited briefs/shortlists; paid tiers unlock more brief volume, deeper signals, SLA-backed reporting; overage model on verified campaigns.
- Integrations: channel/social connectors for verification signals (priority: LinkedIn/B2B, micro/nano sources); reporting exports (CSV/JSON) and webhooks for campaign events; email/notifications for briefs/intros/alerts.
- Compliance posture: baseline privacy + data integrity; SOC2-lite controls (audit logs, access review, backups); no regulated data expected.

### Required Sections (from project type)
- Tenant model: shared multi-tenant; option to add hybrid/single-tenant later for sensitive brands.
- RBAC/permission structure: Brand Admin (billing, team, briefs, approvals); Brand Member (create/edit briefs, review shortlists); Creator (manage profile, verification, chat, reporting); Fan (badge flow); Ops/Admin (flags, overrides, rules, degraded mode).
- Subscription tiers: usage caps by briefs/shortlists/campaigns; feature gating (advanced signals/reports, alerts, SLA); overage pricing on verified campaigns.
- Integration list: social/identity for verification; reporting/webhooks; notifications.

### Implementation Considerations
- Guardrails: enforce tenant scoping in every data access; ensure “never show unverified as verified.”
- Performance: p95 < 2s for verification/shortlist/report flows; designed for multi-tenant scale.
- Extensibility: plug-in model for signal providers; degraded-mode handling when providers hiccup; admin rule updates and alerting.

## User Journeys

**Journey 1: Lina (Micro-creator) – Earning a Verified Profile That Brands Trust**  
Lina, a micro-creator with a niche B2B audience, is tired of brands doubting her reach quality. She signs up, connects social accounts, and finishes verification in one sitting. The platform surfaces her authenticity score, flags clean audience segments, and highlights her best-performing posts. Her “aha” is seeing a trust badge and transparent signals that flip brand doubt. A brand brief arrives the next day; Lina’s verified profile gets shortlisted automatically. If verification flags something, she gets a clear recovery path to fix it. She chats through the platform, shares a transparent report, and secures a paid collab without back-and-forth on proof.

**Journey 2: Mark (Brand Marketer) – Booking a Fraud-Resistant Campaign Quickly**  
Mark needs credible creators for a product launch and fears bot-driven reach. He posts a brief with target audience/vertical. Within hours, he gets a shortlist of ≥5 verified matches, each with authenticity signals and fraud/deepfake flags. If the system can’t find five perfect matches, it returns the best partial shortlist with warnings. His trust moment is the report: transparent signals, fraud flags, and audit trail. He requests intros; the platform tracks approvals and generates a campaign report that proves real engagement. Mark repeats with new briefs after seeing reduced invalid engagement.

**Journey 3: Jess (Fan/Follower) – Proving She’s Real to Unlock Perks**  
Jess follows Lina and wants early access drops. She taps “Get trusted badge,” verifies in a few minutes, and sees a badge on her interactions. Her engagement counts toward verified metrics; she gets prioritized for giveaways and Q&As. If verification fails, she gets a gentle, guided recovery path without losing face.

**Journey 4: Priya (Platform Ops/Admin) – Keeping Verification Clean and Fast**  
Priya monitors verification queues and fraud alerts. She reviews edge cases the system flags (possible bot/fake clusters), overrides or confirms, and updates rules. She audits that no unverified profiles are shown as verified, checks uptime/perf dashboards for verification and shortlist services, and ensures reports include traceable signals. If verification signal providers hiccup, she can trigger degraded mode and communicate warnings while keeping guardrails intact.

### Journey Requirements Summary
- Onboarding & Verification: Single-session creator verification; transparent authenticity scores; fan badge flow <5 minutes; recovery paths for failed checks.
- Matching & Briefs: Brand brief intake; automated verified shortlists (≥5 matches) with signals; fallback to partial shortlist with warnings when needed; intro and approval flow; campaign execution tracking.
- Reporting & Trust UX: Authenticity/fraud/deepfake flags visible in profiles and reports; audit trail of signals; easy-to-share reports for brands; warnings surfaced when in degraded/partial modes.
- Diversification & Risk: Basic diversification guidance in reporting (channel risk cues); alerts on fraud/risk anomalies (growth phase).
- Ops/Admin: Review queue for flagged cases; rule updates; audit that nothing unverified is marked verified; perf/uptime monitoring for verification and shortlist flows; degraded-mode controls when signal providers hiccup.
- Edge Handling: Failure/recovery guidance in verification; guardrails against showing unverified as verified; fallback if shortlist can’t meet target (partial + warnings).

## Project Scoping & Phased Development

### MVP Strategy & Philosophy
**MVP Approach:** Problem-solving + platform foundation (trust-first verification, automated shortlist, transparent reports).  
**Resource Requirements:** Lean squad (PM, UX, 2-3 engineers covering FE/BE, part-time data/ML for signals, part-time QA/ops).

### MVP Feature Set (Phase 1)
**Core User Journeys Supported:**
- Creator verifies once, gets authenticity score/badge, is shortlist-eligible.
- Brand posts brief, receives verified shortlist (with partial-list warnings if needed), requests intros, runs a verified campaign with a transparent report.
- Fan earns trusted badge quickly; engagement counts toward verified metrics.
- Ops/Admin handles flagged edge cases, rule updates, degraded mode, and ensures “never show unverified as verified.”

**Must-Have Capabilities:**
- Single-session creator/fan verification with transparent signals; recovery paths.
- Brief intake → verified shortlist (≥5 or partial with warnings); intro/approval flow; campaign tracking.
- Trust UX: badges, signals in profiles/reports; audit trail; fraud/deepfake flags.
- Multi-tenant scaffolding, RBAC (Brand Admin/Member, Creator, Fan, Ops), basic notifications, exports/webhooks.
- Guardrails and degraded-mode controls; correctness checks (no unverified shown as verified).

### Post-MVP Features
**Phase 2 (Growth):**
- Deeper diversification playbooks and channel-risk insights; automated fraud/risk alerts.
- SLA-backed reporting; richer collab workflows; faster approvals; perks/reputation tiers.
- Expanded integrations and more granular permissions/quotas.

**Phase 3 (Expansion):**
- Standardized authenticity score across ecosystem; third-party audits.
- Advanced orchestration and automated failover when platform risks change; broader markets.

### Risk Mitigation Strategy
**Technical Risks:** Signal accuracy/false positives; mitigation—ops review queue, backtests vs known-good/bad cohorts, audit trails, degraded mode if providers fail.  
**Market Risks:** Brand trust/adoption; mitigation—A/B reports with signals vs. without, measure shortlist acceptance and invalid-engagement reduction, fast TTV (<24h shortlist).  
**Resource Risks:** Team bandwidth; mitigation—lean MVP slice (verification + shortlist + reports), defer advanced playbooks/SLA, offer manual ops support early.

## Functional Requirements

### Verification & Identity
- FR1: Creators can complete a verification flow to generate an authenticity score and badge.
- FR2: Fans can complete a lightweight verification flow to earn a trusted badge.
- FR3: The system can surface verification signals (e.g., authenticity indicators, fraud/deepfake flags) on creator profiles and campaign assets.
- FR4: Users can recover from failed verification with guided remediation steps.
- FR5: The system can prevent unverified profiles from being displayed as verified in any context.

### Briefs, Matching, and Intros
- FR6: Brand users can create and submit campaign briefs with target audience/vertical and objectives.
- FR7: The system can generate a verified shortlist for a brief, including authenticity signals for each match.
- FR8: The system can return a partial shortlist with warnings when sufficient qualified matches are unavailable.
- FR9: Brand users can request and approve intros with shortlisted creators.
- FR10: Creators can accept/decline intro requests and manage availability.

### Campaign Execution & Tracking
- FR11: Brand users can initiate a verified campaign from an approved intro.
- FR12: Creators can confirm participation details for a campaign.
- FR13: The system can track campaign status and key milestones (e.g., approved, in-progress, completed).
- FR14: Brand users and creators can exchange campaign updates and assets within the platform.

### Reporting & Trust UX
- FR15: Brand users can view campaign reports that include authenticity scores, fraud/deepfake flags, and engagement quality signals.
- FR16: Brand users can export campaign reports and underlying signals (e.g., CSV/JSON).
- FR17: The system can display diversification/risk cues (e.g., channel risk guidance) within reports or briefs.
- FR18: The system can show warnings when operating in degraded mode (e.g., partial signals) and annotate reports accordingly.

### Accounts, Roles, and Tenancy
- FR19: Organizations can operate in a shared multi-tenant environment with tenant-scoped data isolation.
- FR20: Brand Admins can manage billing, team membership, and permissions.
- FR21: Brand Members can create/edit briefs and review shortlists per assigned permissions.
- FR22: Creators can manage their profile, verification status, and collaboration settings.
- FR23: Fans can manage their trusted badge status and related profile indicators.
- FR24: Ops/Admin users can review flags, apply overrides, and update rules.
- FR25: The system can enforce role-based access across briefs, intros, reports, and admin actions.

### Notifications & Integrations
- FR26: Users can receive notifications for key events (e.g., shortlist ready, intro requests, verification outcomes, alerts).
- FR27: The system can deliver webhook callbacks for key events (e.g., shortlist generated, campaign status changes).
- FR28: Users can export or download data relevant to their role (e.g., reports, shortlists).

### Admin, Risk, and Quality Controls
- FR29: Ops/Admin can review flagged verification cases and approve/deny with auditability.
- FR30: Ops/Admin can adjust verification rules and thresholds.
- FR31: The system can enter degraded mode when signal providers are unavailable and maintain guardrails.
- FR32: The system can maintain an audit trail for verification signals, overrides, and report annotations.
- FR33: The system can surface alerts for anomalies (e.g., suspected fraud or signal gaps) to Ops/Admin.

### Subscription & Usage (if applicable to go-to-market)
- FR34: The system can enforce usage limits (e.g., briefs, shortlists, campaigns) by plan/tier.
- FR35: Brand Admins can view current plan, usage, and overage status.
- FR36: The system can gate advanced features (e.g., deeper signals, SLA-backed reporting) by plan/tier.

## Non-Functional Requirements

### Performance
- NFR1: Verification, shortlist generation, and report retrieval user actions complete with p95 < 2s under normal load; degraded mode must surface warnings when signals are partial.
- NFR2: Shortlist generation SLA: initial result (verified or partial with warnings) within 24h of brief submission.

### Reliability & Availability
- NFR3: Core verification and shortlist services target 99.9% uptime; degraded mode engages when signal providers fail but must keep guardrails (never show unverified as verified).
- NFR4: Audit trail availability for verification signals, overrides, and report annotations must persist across failures and be retrievable by Ops/Admin.

### Security & Privacy
- NFR5: All data in transit and at rest is encrypted; access is role-scoped (tenant isolation enforced).
- NFR6: Verification signals and reports are only accessible to authorized roles; sensitive signals are not exposed to unauthorized users.
- NFR7: Baseline privacy posture (no regulated data expected): data minimization for signals; least-privilege access; audit logging for admin/ops actions.

### Scalability
- NFR8: Support growth to low-thousands of concurrent brand/creator sessions and 10x growth in shortlist/report volume without >10% performance degradation over baselines.
- NFR9: Plug-in model for signal providers allows adding/removing providers without downtime to verification flow; degraded mode handles provider outages.

### Accessibility & UX Quality
- NFR10: Core user flows (verification, brief submission, shortlist review, report viewing) meet WCAG 2.1 AA for perceivable/operable/understandable requirements.
- NFR11: Trust UX elements (badges, warnings, signals) remain visible and distinguishable in high-contrast modes and with assistive tech.

### Observability & Operations
- NFR12: Monitoring covers verification throughput, shortlist SLA, degraded-mode events, and anomaly alerts; alerting routes to Ops/Admin.
- NFR13: Configuration changes (rules/thresholds) are auditable and revertible; Ops/Admin can activate degraded mode and see its status.
