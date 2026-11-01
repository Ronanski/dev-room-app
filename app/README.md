# Dev Room desktop app (Windows)

A portable Windows app that shows your Claude Code sessions as characters in the café office, live.

## How it works

```
Claude Code sessions on your PC
   │  hooks (SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop, SessionEnd)
   │  curl POST → http://127.0.0.1:4317/hook
   ▼
DevRoom.exe  (Tauri window + local server on port 4317)
   │  /api/state, polled every second
   ▼
Phone on the same Wi-Fi  (http://<PC IP>:4317/?t=<pairing token>, from the QR code)
```

- **Status from hooks:**
  - `UserPromptSubmit` and `PreToolUse` mean working; the tool decides the bubble ("Ine-edit ang trip.ts.").
  - A `Notification` (permission prompt) means the session needs you.
  - `Stop` means the turn ended. If the last assistant message ends in a question, it needs you; otherwise it is on break.
  - `SessionEnd` removes the session.
- **Session names** come from the transcript (`/rename` title or summary), otherwise the folder name. You can rename sessions in **Piliin ang team**. Use `ROLE: task` (for example `CODE FIXER: logic`) to get the role, and pick the leader there.
- **Hooks setup:** the **I-set up ang hooks** button merges six hooks into `%USERPROFILE%\.claude\settings.json`. It keeps your other settings and hooks, writes a backup first, and is safe to press again. Restart open Claude Code sessions afterwards.
- **Phone:** **I-connect ang phone** shows a QR code with a pairing token. Without the token, other devices on the network get nothing. Hooks are accepted only from the PC itself.
- **State** (team, names, last known sessions, token) lives in `%APPDATA%\DevRoom\state.json`.

## Office life

- **Characters:** every session is a woman with her own hairstyle (long, ponytail, bob or bun) and outfit (skirt, dress or pants), picked from her session id.
- **Seats:**
  - Working means seated at the PC.
  - After a turn ends, a session stays at her desk for 2 to 5 minutes before taking a break.
  - On break, she comes back to her desk now and then to check for new work.
- **Break activities:**
  - Under 10 minutes: coffee, CR, stretching, phone, chatting, darts and billiards.
  - Longer breaks add TV, chess and eating.
  - After 30 minutes she naps in the lounge.
  - Billiards and chess pair players up.
- **The leader:**
  - Stays seated at the LEAD desk.
  - After a long idle, she walks the floor checking on each agent ("Kamusta ang ...?") and they answer.
  - When she needs you, she goes to OPISINA MO.
- **You (Boss):**
  - Sit at your desk.
  - Walk out to meet anyone who needs you.
  - After you give a session a prompt, go and watch her work for a minute or so.
  - After 15 minutes with no prompt: coffee, CR or stretching.
- **The guard** plays ML on her phone, dozes off sometimes, and wakes up to greet new sessions.
- **Bubbles** stay up for at least 6 seconds, longer for long text. Lines come from large pools per state and role, plus the session's real task, with no immediate repeats. The A− and A+ buttons set the text size per device.

## Away from home (Tailscale)

Install Tailscale on the PC and the phone and sign in to the same account. Dev Room then puts the PC's Tailscale address in the pairing QR next to the Wi-Fi address. The Android app and its background watcher try home Wi-Fi first and use Tailscale when away.

## Limits of this version

- Replying from Dev Room is not possible yet. Hooks cannot inject messages into a running session, so answer in the Claude desktop app.
- Only sessions running on this PC report in. Cloud sessions on claude.ai/code need a public endpoint, which is planned.
- Sessions that were already open before the hooks were installed appear after they are restarted.

## Build

The **Windows app** GitHub Action builds `DevRoom.exe` on every push that touches `app/`. It publishes the files to a new GitHub release for that build; builds from `main` become the Latest release.

Local build on Windows, with Rust and Node installed:

```
cd app
npm install
npx tauri build --no-bundle
```

The exe needs the WebView2 runtime, which ships with Windows 10 and 11.

`ui/index.html` is the app's UI and is now edited directly; it started from `mockup/dev-room-live.html`.

## Auto update

Every push to `main` (and, until `main` exists, to this PR's branch) builds and publishes `DevRoom.exe`, `index.html` and `version.json` to a new GitHub release, `v<VERSION>-build.<N>`. Builds from `main` become the Latest release, which installed apps update from; other branches publish pre-releases.

The running app checks `version.json` every 3 minutes.
- **UI update:** when the UI's API version (`api-version.txt`) is unchanged, the new `index.html` is cached in `%APPDATA%\DevRoom\ui` and served at once. Open windows and paired phones reload themselves.
- **App update:** the new `DevRoom.exe` is downloaded next to the running one and swapped in, and the old copy becomes `DevRoom.old.exe`. A green bar then offers **I-restart ngayon**.

Bump `api-version.txt` whenever the UI starts depending on a server change. An older exe then waits for its own update instead of loading a UI it cannot serve.
