# Threat model and release gates

Assets: files, profile, credentials, network identity, clipboard, OS privilege and displayed origin. Adversaries: malicious sites, hostile local HTML, compromised content or updates, malicious extensions and network attackers. Entry points: HTML/CSS/JS, URLs, redirects, IPC, fonts/images, downloads and packages.

M0 parses local files in the UI process. Its small parser is incomplete and not security-reviewed. It must not fetch remote pages or execute JavaScript. Files are size-bounded, UTF-8 and explicitly chosen by the user. Unknown active tags are inert, with script regression tests. This does not make hostile local files safe.

Before remote content: bounded maintained parser with fuzzing; low-privilege renderer sandbox; brokered network/files; validated IPC and quotas; per-site process assignment; origin/CORS/CSP; partitioned cookies; maintained TLS; spoofing and download tests; update signing and incident response. Each feature needs abuse cases, tests and rollback.

Parser crash -> fuzzing and renderer crash containment. Cross-origin theft -> site isolation plus origin/storage policy. Local file theft -> broker only. TLS interception -> validated certificates and error page. Malicious downloads -> broker and provenance UI. Update compromise -> signatures and key controls. Exhaustion -> CPU/memory/input/IPC quotas.
