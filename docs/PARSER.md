# M1a parser boundary

HTML5 tokenization and tree construction use html5ever. Patapim keeps its own filtered Node tree for style/layout/painting. Full HTML5 parsing does not imply full CSS/layout compatibility.

Versions checked on 2026-11-05: html5ever latest is 0.40.1. The available markup5ever_rcdom 0.39.0+unofficial requires html5ever 0.39; the compatible pair is pinned exactly and Cargo.lock pins transitives. RcDom is explicitly an unsupported test-oriented intermediate tree. It is temporary: replace it with Patapim's own bounded TreeSink before expanding content trust. Sources: https://docs.rs/html5ever/ and https://docs.rs/markup5ever_rcdom/ .

Input larger than 1 MiB is rejected before parsing. The entire intermediate tree is checked iteratively for 20,000 nodes and depth 64, including hidden/template subtrees. Exceeding a limit returns a typed error; no partial document is displayed. These are post-parse DOM limits, not hard bounds on parser allocations or CPU. The file reader reads at most 1 MiB + 1 byte even if the file grows between metadata and reading. The UI accepts absolute drive paths; UNC paths are rejected. Local junctions and reparse points are not a security boundary.

Head metadata, script, style, iframe, object, embed, template, noscript and foreign namespaces do not enter the render DOM. Unknown passive elements retain text through a transparent wrapper. Only exact style attribute names are accepted; no event attributes or URLs are acted upon. Rendering supports the previously documented small tag/CSS subset.

Regression tests cover entities, omitted end tags, quoted delimiters, exact attribute names, inert subtrees, node/depth/input limits, plus 256 deterministic malformed-input mutations. A separate cargo-fuzz target exercises parser, conversion, layout and destruction. CI runs a 30-second smoke campaign; this is not a security audit or sufficient fuzz coverage for untrusted input.

Run on a host with nightly Rust and cargo-fuzz 0.13.2:

    cargo +nightly fuzz run html -- -max_total_time=30 -max_len=8192 -rss_limit_mb=1024

Next gates: bounded custom TreeSink, longer corpus-based campaigns, CSS parsing/fixtures and process resource limits. Network and arbitrary JS remain disabled.
