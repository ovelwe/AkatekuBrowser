# AkatekuBrowser

Rust browser research project. **M0 is a local HTML viewer**, not a safe or compatible web browser. Patapim owns its DOM subset, style resolution, block layout and display list. Win32/GDI presents the display list and rasterizes text. No Chromium, Gecko, WebView or CEF is used.

## Run (Windows x64)

```powershell
cd D:\AkatekuBrowser
cargo run -p akateku
```

Use `Ctrl+T` for a new tab and `Ctrl+L` for the address bar. Enter a local UTF-8 `.html` path of at most 1 MiB, or `akateku:welcome`. Network and JavaScript are disabled. The status strip shows parse/layout time, node count and display command count. These are diagnostics, not performance claims.

Graphite M0 uses a charcoal, slate and mint palette with antialiased rounded tabs, controls and page surface. The Home button returns to the welcome page. This is a first visual direction; theme tokens, accessibility and HiDPI work are scheduled for later milestones.

## M0 contract

Tags: `html body div p h1 h2 h3 span strong em ul li br`. Entities: `&lt; &gt; &quot; &amp;`. Inline CSS only: six-digit `color`, `background`/`background-color`, pixel `font-size`, `margin`, `padding`. Basic vertical flow and approximate word wrapping. There is no CSS cascade, images, links, script execution, accessibility tree or origin isolation. GDI shapes text, while layout estimates its width, so text can overflow. Local files must be treated as trusted input.

## Development

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

M1a uses pinned html5ever 0.39.0 and markup5ever_rcdom 0.39.0+unofficial for HTML5 tree construction before converting to Patapim's own retained DOM. The library builds cross platform; the window is Windows only. See [parser limits](docs/PARSER.md), [specification](docs/SPEC.md), [ADRs](docs/ADR.md), [threat model](docs/THREAT_MODEL.md) and [roadmap](docs/ROADMAP.md).
