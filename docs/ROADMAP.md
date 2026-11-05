# Milestones and gates

| Milestone | Deliverable | Exit criterion |
|---|---|---|
| M0 | Local viewer and Graphite shell | Window, tabs/address, local HTML, tests, CI, metrics; no network/JS |
| M1 | Bounded HTML/CSS core | Conformance fixtures, parser fuzzing, documented limits |
| M2 | Text/layout correctness | Shaping, box model, clipping, HiDPI, reference screenshots |
| M3 | Incremental engine and X-Ray | Stable IDs, dependency invalidation, dirty regions, traces; parity with full rebuild |
| M4 | Process boundary and sandbox | Brokered renderer, low privilege, IPC fuzzing, crash recovery |
| M5 | Secure network/navigation | TLS, origin/CSP/CORS, cookies, permissions; review before external URLs |
| M6 | Web platform and JS subset | Isolated maintained runtime, DOM bindings, compatibility suite, quotas |
| M7 | Browser UX | History, bookmarks, downloads, accessibility, themes, keyboard support |
| M8 | Compositor/performance | Tiles, CPU/GPU selection, background policy, fair Firefox/Chromium comparison |
| M9 | Beta and Linux | Signed updates, Linux sandbox parity, security response, compatibility bar |

M1a implemented: HTML5 parsing through a pinned compatible library pair, conversion to Patapim DOM, explicit input/tree/depth errors, bounded file reads, malformed-input regression tests and a short fuzz CI target. M1 remains open: the temporary RcDom intermediary must be replaced by a bounded custom TreeSink, CSS needs conformance fixtures, and fuzz campaigns need broader corpora and longer runs. Accurate text measurement and reference screenshots follow in M2. Keep network navigation disabled; add renderer separation before expanding content trust.

Risks: web compatibility scope, parser/layout security bugs, platform-specific sandbox gaps, GPU overhead, font/accessibility complexity and security maintenance capacity. Every milestone must measure its own claim.
