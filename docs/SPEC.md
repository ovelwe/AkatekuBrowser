# Engineering specification

Windows x64 first; Linux second. Priorities: security and correctness, usability and compatibility, then measured CPU/RAM/GPU/energy improvements. Firefox-level ease of use is a long-term UX goal, not an M0 claim.

## Target architecture

Browser process: Graphite chrome, tabs, navigation, profile, history/bookmarks/downloads, permission prompts, updates and process supervision. Network service: HTTP/TLS and partitioned cookies. Renderers: bounded IPC, no arbitrary filesystem/network, per-site assignment, origin enforcement and OS sandbox. Cross-origin frames require isolated processes. A renderer crash must only terminate its tab.

Patapim: bounded HTML parser -> DOM -> CSS cascade/computed style -> box tree -> incremental layout -> display list -> tiled raster/dirty-region compositor -> accessibility tree. Stable node IDs and generation-based dependencies identify the smallest affected style/layout/paint region. Geometry changes propagate through dependent flow siblings. Caches have explicit budgets. Scheduler sleeps when idle, budgets foreground frames and throttles background tabs based on measured time/CPU. Patapim X-Ray records invalidation reasons, stage timings, tile work, cache behavior and memory. Deterministic replay follows. CPU/GPU paths are selected by measured workload with CPU fallback.

## Distinct security policies

Process/site isolation governs process assignment, IPC and crash/resource containment. OS sandbox denies direct file/network access. Origin policy covers same-origin, CORS, CSP, mixed content and navigation. Cookies require partitioning, SameSite, Secure/HttpOnly and bounds. Permissions require origin-specific grant, expiry and revocation. TLS uses a maintained library and system trust; certificate errors fail closed. Updates need signed releases, vulnerability triage and an emergency release process. No custom cryptography.

## Graphite UX target

Token-based light/dark themes; keyboard-friendly tabs and omnibox; history, bookmarks and downloads; clear site permissions; HiDPI; accessibility tree. Chrome and page theme state remain separate.

## Performance protocol

Measure cold/warm startup, idle CPU and wakeups, steady/peak RAM, scroll latency, frame-time percentiles, GPU use and energy where available. Use identical hardware, scaling, power mode, content snapshots, cache state and tab count for Akateku, Firefox and Chromium. Report median and variance across at least ten runs. Synthetic microbenchmarks alone cannot support superiority claims.

## M0 acceptance

Real Win32 window, tabs/address input, local-file navigation, own DOM/style/block layout/display commands, metrics, tests and CI. Local `.html` UTF-8 files are capped at 1 MiB. Parser, text measurement, clipping, keyboard and accessibility gaps block untrusted content.
