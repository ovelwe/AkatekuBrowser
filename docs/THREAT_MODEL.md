# Threat model and release gates

Assets: files, profile, credentials, network identity, clipboard, OS privilege and displayed origin. Adversaries: malicious sites, hostile local HTML, compromised content or updates, malicious extensions and network attackers. Entry points: HTML/CSS/JS, URLs, redirects, IPC, fonts/images, downloads and packages.

M1a parses local files in the UI process using html5ever and a temporary RcDom intermediary. The integration is not security-reviewed. It must not fetch remote pages or execute JavaScript. Files are size-bounded, UTF-8 and explicitly chosen by the user. Active subtrees are filtered before rendering. Node/depth limits apply after the intermediary is built; they do not constrain parser CPU or peak memory. This does not make hostile local files safe.

Before remote content: bounded maintained parser with fuzzing; low-privilege renderer sandbox; brokered network/files; validated IPC and quotas; per-site process assignment; origin/CORS/CSP; partitioned cookies; maintained TLS; spoofing and download tests; update signing and incident response. Each feature needs abuse cases, tests and rollback.

Parser crash -> fuzzing and renderer crash containment. Cross-origin theft -> site isolation plus origin/storage policy. Local file theft -> broker only. TLS interception -> validated certificates and error page. Malicious downloads -> broker and provenance UI. Update compromise -> signatures and key controls. Exhaustion -> CPU/memory/input/IPC quotas.
