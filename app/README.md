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

## Limits of this version

- Replying from Dev Room is not possible yet. Hooks cannot inject messages into a running session, so answer in the Claude desktop app.
- Only sessions running on this PC report in. Cloud sessions on claude.ai/code need a public endpoint, which is planned.
- Sessions that were already open before the hooks were installed appear after they are restarted.

## Build

The **Windows app** GitHub Action builds `DevRoom.exe` on every push that touches `app/`. It publishes the file to the `dev-room-latest` pre-release.

Local build on Windows, with Rust and Node installed:

```
cd app
npm install
npx tauri build --no-bundle
```

The exe needs the WebView2 runtime, which ships with Windows 10 and 11.

`ui/index.html` is generated from `mockup/dev-room-live.html`, so keep the two in step when changing the office view.
