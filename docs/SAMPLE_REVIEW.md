# Sample Review — real-world output

This is a full, unedited review produced by PRD Co-Pilot on a real
internal PRD. Shared here so leadership can see the exact format that
reviewers and engineers will receive.

---

## PRD Review: T-160014 — Lottie Animation For Unlocked Feature

**Score: 24/100** — ❌ **NEEDS REVISION** (threshold: 95)
**Reviewer:** PRD Co-Pilot (acting as senior PO)
**Date:** 2026-04-17
**Wiki:** [page 76096147](https://wiki.tuntun.co.id/pages/viewpage.action?pageId=76096147)

### PO view, one paragraph

This is a 1–2 sprint delighter (animation on unlock) masquerading as
an implementation-ready spec. We've got a Designer + Figma +
screenshots + LCMP — that's the fun half. The missing half is
everything an engineer and a QA need to stop asking questions: *why
we're doing it*, *what success looks like*, *what the flow is when
the user enters from the second path*, *what happens on error /
offline*, *what the rollout gate is*. At 24/100 this should not go
into a sprint. Fixable in a half-day of PM work though — the content
is there, it just isn't structured.

### Section Checklist

| # | Section | Status | Points | Notes |
|---|---|---|---|---|
| 1 | Metadata | Incomplete | -2 | Designer + Figma present. Missing: Status, Owner, Version, Changelog, Urgency, Request Type |
| 2 | TL;DR | MISSING | -5 | No summary — reader drops straight into "Scope Requirement" |
| 3 | Background & Problem | MISSING | -10 | No *why*. Why now? What user/business pain? |
| 4 | Objectives & Success Metrics | MISSING | -12 | No measurable goal, no KPI. "Unlock awareness +X%"? Undefined |
| 5 | Scope (In/Out) | Incomplete | -3 | In-scope listed. Out-of-Scope not explicit |
| 6 | User Stories | MISSING | -7 | No persona framing. Who benefits? |
| 7 | Functional Requirements | Incomplete | -6 | Rules partial. Persistence missing. Edge cases absent |
| 8 | Design Reference | Incomplete | -3 | Figma ✓, screenshots ✓. Missing figure X.N labels; no Lottie asset |
| 9 | User Flows / Journey | MISSING | -8 | Mutual-exclusion rule implies a state machine — nowhere drawn |
| 10 | Acceptance Criteria | MISSING | -15 | No Given/When/Then |
| 11 | Risks & Open Questions | MISSING | -5 | Device-local-date is a known risk — unflagged |

### Blockers (P0 — must fix before a sprint can plan this)

1. **Acceptance Criteria** — name the 4 canonical scenarios: first unlock
   shows animation · same-day re-entry does NOT show · next calendar
   day re-entry DOES · mutual-exclusion (market → secondary) does NOT.
   Given/When/Then each.
2. **Objectives & Success Metrics** — pick one primary KPI. Candidates:
   "unlock-engagement rate +X%", "Premium Feature CS tickets –Y%",
   "secondary-page entry rate +Z%".
3. **Background & Problem** — one paragraph on *why this now*.
4. **User Flows** — draw 2 flows: (a) market-page first-trigger →
   secondary-page no-trigger · (b) secondary-page first-trigger → entry
   from different secondary → no re-trigger.

### Quality Issues (P1 — should fix)

1. **Data & Update** — specify persistence key, reset rule, re-install
   behaviour.
2. **Clock-skew risk** — flag as Risk with mitigation.
3. **Out-of-Scope list** — add: accessibility, Android parity, dark-mode.
4. **Figure labels** — link rules ↔ designs.
5. **Lottie asset** — link actual `.json` or handoff.

### Suggestions (P2)

1. Changelog table in Metadata.
2. Event Tracking lightweight table.
3. Clarify the "Premium Features Unlocked" state shown above blurred
   content in screenshots — is that the intermediate state or final?

### Engineer FAQ — pre-flight questions

Resolve **OPEN** items before the engineering assessment meeting.
That session should debate edge cases, not hunt for information.

| Category | Question | Status | Notes |
|---|---|---|---|
| Data & Persistence | Where is "already shown today" state stored? | ❌ OPEN | Client `UserDefaults` vs server flag vs both |
| Data & Persistence | What key resets the "shown today" flag? | 🟡 PARTIAL | "Next day" per device — but implementation key + reset trigger not specified |
| Data & Persistence | Behaviour on app reinstall / logout → login? | ❌ OPEN | Likely resets, but not stated |
| State & Concurrency | User enters market + secondary in same session — which wins? | ✅ ANSWERED | Mutual-exclusion rule: first one fires, suppresses the other |
| State & Concurrency | What if animation is interrupted mid-play? | ❌ OPEN | Replay? Skip? Mark as shown? |
| State & Concurrency | Two devices, same account — each shows once or sync? | ❌ OPEN | Device-local-date implies independent — needs explicit OK |
| Error & Offline | Offline at unlock moment — still animate? | ❌ OPEN | Unlock is client-side visual; assume yes but confirm |
| Error & Offline | Lottie asset fails to load — fallback? | ❌ OPEN | Degrade path not specified |
| Platform & Device | iPad layout? | ❌ OPEN | Screenshots are phone-only |
| Platform & Device | Min iOS version? | ❌ OPEN | Lottie-ios has a floor; pin it |
| Platform & Device | Dark mode variant? | ❌ OPEN | Light mode only in screenshots |
| Platform & Device | Reduced-motion / VoiceOver? | ✅ ANSWERED | Out of scope for v1 (PM decision) |
| Platform & Device | Device clock-skew tolerance? | 🟡 PARTIAL | Device-local-date chosen — risk accepted but not written |
| Integration Contracts | What signals "NEW unlock" vs "already unlocked"? | ❌ OPEN | Server flag vs pure client inference? |
| Integration Contracts | Lottie asset URL / bundle location? | ❌ OPEN | Figma has static screens only |
| Observability & Rollout | Analytics event on animation shown? | ❌ OPEN | Recommend `UnlockedAnimation_Shown` with `module` + `entry_point` |
| Observability & Rollout | Feature flag name + default? | ❌ OPEN | Need kill-switch for older-device jank |
| Observability & Rollout | Rollback trigger + owner? | ❌ OPEN | Crash-free rate? Performance regression SLA? |

**FAQ summary:** 14 OPEN · 2 PARTIAL · 2 ANSWERED (18 total).

For this PRD the FAQ IS the real handoff document. 14 OPEN items
means the engineering assessment would spend ~90 minutes on
clarifications before anyone could estimate. Closing these first
turns that into a 30-minute edge-case session.

### Strengths

1. **Clear module scope** — Top Pick Today, Top Undervalued, Tuntun
   Portfolio is concrete and unambiguous.
2. **LCMP completeness** — all 3 unlock-state strings provided in
   EN/ID/ZH, with keys.
3. **Mutual-exclusion insight** — the market vs secondary suppression
   rule is subtle and important; catching it pre-build saves a real
   bug.
4. **Figma link works** and the 3 screenshots render — reviewers can
   actually inspect the design.
5. **Compact** — small feature, small PRD. Right instinct; just needs
   structure.

### Action Items

| Priority | Item | Owner | Effort |
|---|---|---|---|
| P0 | Add Acceptance Criteria (4 scenarios) | PM | 30 min |
| P0 | Add Objectives + 1 KPI + measurement window | PM | 20 min |
| P0 | Add Background (1 paragraph + data point) | PM | 15 min |
| P0 | Draw User Flows for both entry paths | PM + Design | 1 hr |
| P1 | Add Data & Update (persistence, reset, reinstall) | PM + iOS Lead | 20 min |
| P1 | Add Risks (clock skew) + Out-of-Scope list | PM | 15 min |
| P1 | Link Lottie asset handoff | Designer | 10 min |
| P2 | Add Event Tracking + Changelog | PM | 20 min |

Total PM effort to clear: **~2.5 hours**. Target post-fix: **≥ 95/100**.
