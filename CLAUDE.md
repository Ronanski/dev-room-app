# Dev Room: rules for Claude

Dev Room shows the user's Claude Code sessions as characters in an isometric pixel-art power plant. Each session is a female character; the user appears as the **Boss**. The office has a control room, team supervisor office, the Boss's office, library, tulugan, gate, CR, pantry, lounge, rec room, gym and a basketball half court.

The user's real team is **LOGIC_SIM_PROJECT**: a leader plus subsessions (CODE FIXER, HMI, APP UI/UX, LIVE TESTER, DOCS) working on `Ronanski/icms-master-tool`.

## How to talk to the user

- **Language:** Taglish (Filipino with English terms). Keep replies short and organized, with clear headings or bullets. The user reads on a phone.
- **No links:** never give links (download pages, releases, artifacts). The user explicitly forbade it.
- **One consistent answer:** never say "download this" and then "don't download". Updates arrive by themselves; say exactly what to press, if anything.
- **Be honest about testing:** say plainly what was tested here (Playwright, a local server build) and what was not (the real Windows PC, the Android phone).
- **Find bugs yourself:** do not ask the user to find bugs or to send screenshots for things you can test. Use the app end to end before saying it works.
- **Fix one by one:** fix each problem completely before moving to the next, then report what changed.

## Release flow

1. Work on the designated branch, restarted from `origin/main` after each merge (`git checkout -B <branch> origin/main`).
2. **Before each release:**
   - Bump `VERSION` (semver).
   - Add a `CHANGELOG.md` entry.
   - Add a `NOTES['x.y.z']` entry in `app/ui/index.html`. This is the one-time "Bago sa Dev Room" card, with a **Gawin mo** list of the steps the user must take.
3. Push, open a draft PR, and subscribe to it.
4. When CI is green (windows, android and publish all pass), the user has asked that the PR be marked ready and merged.
5. Main builds publish the Latest release `v<VERSION>-build.<N>`, which carries `DevRoom.exe`, `DevRoom.apk`, `index.html` and `version.json`.
6. **The PC app updates itself:**
   - The UI is hot-swapped right away.
   - The exe is swapped in, then the app restarts by itself while it sits in the tray, or after a countdown when the window is open.
7. **The phone app** shows an **I-update** banner; installing the APK is the only manual step.

## Architecture

- **`app/src-tauri/src/server.rs`:** a `tiny_http` server on port 4317, one thread per request.
  - Claude Code hooks POST to `/hook?devroom=1`: SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, SubagentStop, Notification, Stop and SessionEnd.
  - Hook replies carry chat messages: `additionalContext` on PostToolUse and UserPromptSubmit, or a `decision:block` reply on Stop.
  - The Stop wait (`away` mode) is **off by default**, because a held Stop makes the session look stuck in Claude Code.
  - **Security:**
    - The Host header must be an IP address or `localhost`.
    - A browser `Origin` must be Dev Room itself; the Android shell origins are accepted only from remote callers.
    - Every `POST /api/*` needs the `X-DevRoom` header, and `/hook` refuses requests that carry an `Origin`.
    - Remote callers need the pairing token.
    - `/api/claude-view` needs the per-launch `app_key()`.
  - **Malayo mode** (`tunnel.rs`): cloudflared quick tunnel → loopback port 4318.
    - Every request on that port is treated as remote: token required, hooks and PC-only routes refused.
    - The address is published to the ntfy topic `devroom-<sha256("devroom-relay:"+token)[:16 bytes hex]>`.
    - The phone shell and `WatchService` use it as the last fallback.
  - **Real-time chat:**
    - `tail_transcripts` follows each transcript (every 0.9 s) and pushes prompts and replies to the conversation.
    - `deliver_now` continues an idle session with `claude -p --resume <sid>`, in its cwd and `perm_mode`.
    - Working sessions get messages through the PostToolUse `additionalContext`.
    - The Chat tab is Dev Room's own chat; never replace it with claude.ai.
  - A liveness thread marks a session as working while its transcript keeps being written after Stop.
  - State lives in `%APPDATA%\DevRoom\state.json` and is written atomically.
- **`app/src-tauri/src/main.rs`:**
  - The tray app; closing the window destroys the webview (low resources).
  - Starts with Windows by default.
  - Single instance through `/api/show`.
  - The embedded claude.ai chat is a child webview over the Chat panel. It uses the Tauri `unstable` feature, and its navigation is limited to an allowlist.
- **`app/src-tauri/src/updater.rs`:** polls the Latest release every 3 minutes and does the UI hot swap and the exe swap.
- **`app/ui/index.html`:** the whole UI in one file: canvas map, overlay, panel, chat feed and notes. Edit it directly; `tools/ui-build/` only reproduces older versions.
- **`mobile/`:** the Capacitor Android shell.
  - It shows the office in an iframe, served by the PC.
  - `ClaudeActivity` is the in-app claude.ai screen.
  - `WatchService` sends background alerts.
  - It pairs with `devroom://pair?u&t&a`, with a Tailscale fallback.
- **CI:** `.github/workflows/windows-app.yml`.
  - The APK is signed with the secrets `ANDROID_KEYSTORE_B64` and `ANDROID_KEYSTORE_PASSWORD`; never print them.

## Testing here (no Windows or Android device)

- The full Tauri app cannot build on Linux (no GTK). Compile `server.rs` and `updater.rs` in a standalone crate instead:
  - Copy them into a scratch crate with `tiny_http`, `serde_json`, `local-ip-address`, `qrcode` and `rand`.
  - Add a `main.rs` that calls `server::start(<port>)`.
  - The CI Windows build is the real check for `main.rs`.
- **UI:** use Playwright against that server. Feed it with hook JSON through `curl`.
  - Check:
    - Viewports: 1366×768, 1920×1080, 1280×800 touch and 412×900 phone.
    - Light and dark mode.
    - No page errors.
    - No overflow.
    - No overlapping labels.
  - Look at the screenshots.
- **Hook events:** test every event and flow end to end before saying something works.

## Things the user cares about

- **Real-time sync:** what a character does must match its session; working means at a desk.
- **Light on resources:** 60 fps, with no lag or blinking on the PC, phone or tablet.
- **Life in the office:** realistic, non-repetitive office life, plus the off-duty crew (Bea, Mika, Trish, Joy).
- **Real chat:** Dev Room's own chatbox, in real time and with no waiting. Chatting there must feel the same as in the Claude app. Do not embed claude.ai instead.
- **Polish:** complete and polished. The user dislikes "3rd world" half-done work.

## Backlog

See `PROGRESS.md`.
