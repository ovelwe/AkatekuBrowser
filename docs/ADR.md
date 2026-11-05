# Architecture decisions

## ADR-001: own rendering core
Patapim owns DOM, styles, layout, display list and painting orchestration. Platform APIs may rasterize glyphs or present pixels. This keeps the engine independent while a mature text shaping stack is selected.

## ADR-002: no network or JS in M0
The single-process prototype lacks sandbox and origin enforcement. URL fetching and script execution are gated on isolation, IPC validation and policy work. Local files are explicitly trusted.

## ADR-003: zero external crates for M0
The Cargo registry was unreachable in this environment. M0 uses the standard library and Win32 FFI. Before adopting parsers, shaping, TLS and HTTP crates, verify current releases and security posture, then commit the lockfile. No custom cryptography.

## ADR-004: display list boundary
Patapim emits platform-neutral rectangle/text commands; Windows presents them through GDI and uses system GDI+ for antialiased rounded Graphite controls. This is a presentation API, not a browser rendering engine. M0 rebuilds on viewport resize. Dependency-driven incremental invalidation is an M3 gate.

## ADR-005: event-driven shell
Win32 sleeps between messages, with no periodic repaint loop. Adaptive scheduling follows workload measurement and X-Ray instrumentation.
