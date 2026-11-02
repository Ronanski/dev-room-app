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
7. **The phone app** shows an **I-update** banner. It downloads inside the app; the user presses Android's Install, which is the only manual step.

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
    - **Real-time while busy:** a message still queued after 2.5 s gets an answer from a fork of the session (`quick_answer`: `--fork-session`, plan mode, `disableAllHooks`), shown as ⚡ in the chat; the message stays queued for the session itself. Messages that arrive meanwhile are answered together next; an answer that comes after the session itself answered is dropped (`quick: late`).
    - **Convo lines** carry `src` (hook, tail, quick, sys): a hook line and a transcript line with the same text are paired once; the same text twice from one source is two messages.
    - The Chat tab is Dev Room's own chat; never replace it with claude.ai.
  - Subagent hooks (`agent_id`, or a `subagents` transcript) carry the parent's session id: they only update its detail text while it works, never its status or transcript, so a lead with agents in the background stays free for messages. Prompts Claude Code writes itself (`system_prompt()`, `SYS_PROMPT` in the UI) are not shown as the user's messages.
  - **Progress:** `track_tasks` keeps each session's task list (`tasks`: id, t, a, st) from TodoWrite / TaskCreate / TaskUpdate; the UI's `prog()` drives tile bars, map bars, the Detalye checklist and the report walk to the Boss (`reportToBoss`).
  - **Queue reason:** `queue_why` (answering, step, nocli, busy, soon) is shown under a waiting message (`queueText`).
  - **Plan limits:** `fetch_usage` reads the 5-hour and weekly utilization every 2 minutes with the token in `~/.claude/.credentials.json` (never passed on); `usage` in `/api/state`, shown by `renderUsage`. `DEVROOM_USAGE_URL` points it at a stand-in for tests.
  - **Names:** `read_transcript` and the live reader take `custom-title` (`/rename`, the Claude app) into `custom_title` and `ai-title` / `summary` into `auto_title`; the title is a Dev Room rename (`team.names`), else custom, else auto, else the first prompt. `own_title` (without the Dev Room rename) is what the UI compares rename boxes with. The panel header has ✎ rename (`RENAME`, `saveName`) and a Playroom picker (`roomPicker`, `moveRoom`).
  - **SessionStart** with `source: "compact"` comes mid-turn: it never sets the session idle.
  - Subagent transcripts are recognised by a `/subagents/` folder only (`is_sub_path`), never by a project name.
  - **SessionEnd** with `reason: "other"` from Dev Room's own `claude -p --resume` run is ignored (`ran_recently`: running, or ended under two minutes ago, since the hook can arrive after the run exits); `/exit`, `/clear` and logout always count; a real one sets `ended_at`, and liveness removes the session after a quiet minute (ten for one placed in a playroom). Quick-answer copies (prompt starts with `QUICK_MARK`) are dropped and ignored if their hooks fire.
  - A liveness thread marks a session as working while its transcript keeps being written after Stop.
  - State lives in `%APPDATA%\DevRoom\state.json` and is written atomically.
- **`app/src-tauri/src/main.rs`:**
  - The tray app; closing the window destroys the webview (low resources).
  - Starts with Windows by default.
  - Single instance through `/api/show`.
  - The embedded claude.ai chat is a child webview over the Chat panel. It uses the Tauri `unstable` feature, and its navigation is limited to an allowlist.
- **`app/src-tauri/src/updater.rs`:** polls the Latest release every 30 seconds and does the UI hot swap and the exe swap. It restarts at once in the tray or when minimized; with the window open, after 15 seconds idle.
  - **Install path:** `app_exe()` is fixed at start and is always `DevRoom.exe` (never `current_exe()`, which follows the running image when it is renamed to `DevRoom.old-<build>.exe`). The swap, the restart and the autostart entry use it.
  - The window's place, size and maximized state are kept in `window.json` (`Geom`, `note_geom`, `save_geom`); `make_window` in `main.rs` restores them, and a restart while minimized comes back minimized through `SetWindowPlacement`.
- **`app/ui/index.html`:** the whole UI in one file: canvas map, overlay, panel, chat feed and notes. Edit it directly; `tools/ui-build/` only reproduces older versions.
- **Viewer (in `index.html`):** artifacts and pictures open in a pop-out. `viewerMode()` picks `float` (PC, tablet landscape: draggable, resizable, remembered in localStorage `devroom.viewer`), `card` (tablet portrait), `sheet` (phone or height ≤ 520: swipe down closes) or `full` (⤢).
- **Scenes and units (in `index.html`):**
  - `setScene('office'|'resort')` redraws the background (`drawOfficeBG` / `drawResortBG`), rebuilds the furniture (`buildOffice` / `buildResort`) and re-bakes it.
  - Both scenes share the floor plan: `DESKS`, `SPOTS` and the routes.
  - `computeUnits` uses your playrooms (`TEAM.rooms`: id, name, scene, lead, ids), with unplaced sessions in "Bagong dating".
  - With no playrooms saved, it groups sessions by team or project folder, and the units alternate scenes.
  - The **Mga Playroom** editor is `openTeam` / `renderRooms` / `roomsClick`.
  - New scenes must keep the same desk and spot coordinates.
- **Fullscreen:** the Fullscreen API is not used inside the Android app (`FS_OK`); its WebView collapsed the page. CSS alone fills the screen there.
- **Portrait:** the class `port` on `.app` comes from `screen.orientation` (not the space left by the keyboard). While typing the app has `kbopen` (never `typing`: that is the chat's working-line class).
- **Tablet portrait:** with the panel open on a portrait screen ≥ 641 px wide, the panel docks under the office (grid `office / dock / chat`) instead of overlaying it. The bar above it (`#pSplit`) sets its height (`--chath`, remembered in localStorage `devroom.chath`). While typing (`kbopen`) only the chat shows.
- **Short screens** (height ≤ 520, a phone on its side): the page scrolls and the panel is fixed to the screen's height.
- **Body overflow** is `overflow-x:clip`, never `hidden` (hidden broke the phone's sticky tab bar).
- **Session tiles (`#dock`):** an equal-width grid, up to 4 a row (`dockCols`), with the session's face (`faceURL`, cut from her sprite), one short name and a status dot; lower in portrait and fullscreen.
- **`mobile/`:** the Capacitor Android shell.
  - It shows the office in an iframe, served by the PC.
  - `ClaudeActivity` is the in-app claude.ai screen.
  - `WatchService` sends background alerts.
  - `DevRoom.installUpdate` downloads the APK in the app and opens the Android installer (REQUEST_INSTALL_PACKAGES).
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
