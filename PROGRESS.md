# Dev Room: progress notes

## The idea

The user runs Claude Code projects with one **main session** (the leader) and several **subsessions**. They work like branches, and each one has its own role.
Today the user monitors them one by one.

Dev Room shows them all in one place instead. Each session is a character in a shared office, so the user can see at a glance:

- who is working,
- who needs the user's input,
- what each session is doing right now, and how far along it is.

## Current state (v0.6.5, 2026-11-01)

Shipped and auto-updating:
- **Windows tray app:** hooks give live status, the map is an isometric power plant, there is a chat feed, and the real claude.ai chat is embedded in the panel.
- **Android app:** shows the office, sends alerts, and has an in-app Claude screen.

A full UI/UX, functional and code audit was done, and its fixes went into 0.6.5.

### Not yet verified on real devices
- The embedded Claude view and its sign-in inside the Windows window (Tauri multiwebview).
- The Android app's Claude screen, its sign-in, and the pairing confirmation.
- The auto-restart from the tray on Windows.

### Backlog (ask the user before starting; they pick the order)
1. Sound or vibration when a session needs you, with an on/off switch.
2. Tapping a character opens that session's Claude chat directly.
3. A status count in the window title or tray icon, for example "2 naghihintay".
4. A daily report of what each session did today.
5. Search and filter in the chat feed and the activity log.
6. A Settings panel: bubble size, animation speed, NPCs on or off, away mode.
7. Signed updates. This needs a signing key in the repo secrets, so ask the user first.
8. Approve or deny permission prompts from the app. This needs Claude Code's PermissionRequest hook; check the docs first.
9. Phone: show at most one toast plus a "+N" count, and a compact header everywhere.

## Status (history)

Mockup only. There is no real app yet.

- Mockup: `mockup/dev-room-mockup.html` (single file, runs on sample data)
- **Live view:** `mockup/dev-room-live.html`. It reads the sessions of whichever claude.ai account opens it, through the Claude Code Remote connector (`list_sessions` every 30 s, `get_session` for details), and sends chat messages with `send_message`.
  - Role comes from `ROLE: task` titles. The default leader is the newest title without a colon.
  - "Piliin ang team" lets the user pick the members and the leader. The choice is saved in the browser.
  - If the connector is unavailable, the page shows clearly labelled sample data.
  - Published (owned by the ronan.ece account): https://claude.ai/artifact/GedDDtAx4mCNCTsqj1E6Zv
- Published preview: https://claude.ai/artifact/RcjPyRzi2NLL72JgatPTUw
- **Live data version:** `mockup/dev-room-logic-sim.html` shows the user's real **LOGIC_SIM_PROJECT** team (leader plus five subsessions on `Ronanski/icms-master-tool`), taken from claude.ai/code on 2026-10-31 20:24 UTC.
  - Uses real time, with no random status changes.
  - Each session's question, last status, repo, branch, model, context usage and artifacts come from its session record.
  - `devRoomApply(records)` accepts fresh session records and moves the characters, so a server only needs to push the records.
  - Chat replies are drafts only until the backend can call `send_message`.

## Windows app (in progress)

- `app/` holds a portable Windows app built with Tauri: `DevRoom.exe` plus a local server on port 4317. See `app/README.md`.
- Claude Code hooks on the PC POST events to the app, so status is real time.
  - The app installs the hooks itself, keeping a backup of `settings.json`.
- Phones on the same Wi-Fi open the same office by scanning a QR code that carries a pairing token.
- The GitHub Action **Windows app** builds the exe and the APK and publishes them as release `v<VERSION>-build.<N>`; builds from `main` are the Latest release.
- **Android app (`mobile/`):** a Capacitor app that shows the office from the PC.
  - It pairs through `devroom://pair` from the QR page.
  - A foreground service polls the PC every 15 seconds and sends a notification when a session needs you.
  - It offers its own updates from the same release, and is signed with a fixed key kept in repo secrets.
- Next steps:
  - Replying from the app.
  - Cloud sessions through a public relay.

## Requirements from the user

### Office and characters
- [x] An office with PCs, a pantry, a restroom (CR) and a lounge. It should feel like a normal office with work-life balance.
- [x] **Café-style office:** coffee-shop look with wood floors, brick walls and warm lights. Colors stay muted, because the first mockup was too colorful.
- [x] Each session is a character.
  - When a session is actively working, its character sits at its PC.
  - When it is not working, it walks around, chats with others, eats, uses the CR, or sleeps.
- [x] The **main session is the leader**. It has its own office and delegates to the others.
- [x] **The user has their own office ("Opisina mo")** and appears as the character "Ikaw". Sessions that need input line up outside this office with a raised hand.
- [x] **NPCs:**
  - The Guard greets new sessions at the lobby.
  - HR sends overworked sessions on break and reminds the user who is waiting.
- [x] Rooms are bigger and taller, and bubble text is smaller so the map is not crowded, like the user's reference screenshot.
- [x] **Isometric view** (the user asked to try it after top-down). Every character has a **dark outline** so it doesn't blend into the floor or furniture.
- [x] **Characters and rooms must not look small.** The map is zoomed in by default. It supports drag to pan, + and − buttons, the scroll wheel and an "entire office" button, and it follows a selected character.

### Monitoring and chat
- [x] A **chat box where the user talks to the selected session**. On desktop it sits in a **side panel next to the map**, like the user's reference screenshot (map in the middle, agent panel on the right, agent dock at the bottom).
- [x] The chat shows each session's question with quick replies (Approve / Hindi muna). Sending a message sends the session back to work.
- [x] Panel tabs: Chat, Detalye (what it's doing, how long, where it is, source, recent history), and Lahat ng activity.
- [x] A session dock along the bottom with every session's status and time in that status, plus "add session".
- [x] A notification when a session needs input, and a live event feed on the map.
- [x] Status counts at the top: working, waiting for you, on break, asleep, done or error.

### Text
- [x] Full sentences only, with no cut-off text and no "…". Sentences may be in Taglish.

### Platforms and data
- [ ] **Portable desktop app** and **Android app**. The mockup has a Desktop/Android preview toggle; a real app is not built yet.
- [ ] Must work with **both local CLI sessions and cloud sessions** (claude.ai/code).
- [x] Each session's role comes from its **session name**. The real format is `ROLE: task` (for example `CODE FIXER: LOGIC, ENGINE, PLANT MODEL`). A name with no colon (`LOGIC_SIM_PROJECT`) is the leader.
- [ ] Data source plan, option A: poll claude.ai/code session records (`status_bucket` and `post_turn_summary`). This is already proven by the live version.
  - `WORKING` maps to working, `BLOCKED` to needs input, `REVIEW_READY` to done, `COMPLETED` to idle, and `FAILED` to error.
- [ ] Data source plan, option B: Claude Code hooks POST events to the app's server.
  - SessionStart: a session arrived.
  - UserPromptSubmit and PreToolUse: working, with the current tool.
  - Notification: needs input.
  - Stop: idle.
  - SessionEnd: the session left.

## How each status looks

| Status | Character |
|---|---|
| Working | Sits at the PC with the monitor on. The bubble says what it's doing. |
| Needs input | Lines up outside the user's office with a raised hand and a red ring. A notification also appears. |
| Idle, 0–10 min | Coffee, CR, or walking around. |
| Idle, 10–30 min | Chatting at the water cooler, or eating at the coffee bar. |
| Idle, over 30 min | Sleeping in the lounge. |
| Done | Sits at the desk with a "pa-review" bubble. |
| Error | Sits at the desk with a red, smoking monitor. |

## Open questions

1. ~~What is the real session-name format?~~ Answered: `ROLE: task`, and the leader has no colon.
2. Which stack should the build use?
   - Desktop: Electron or Tauri.
   - Android: Capacitor or React Native.
3. Where should the event server run so that both local and cloud sessions can reach it?
4. How should the app know which sessions belong to one team? The live version uses a fixed list of session IDs. Options are a shared tag, a shared repo, or a name prefix.

## Improvement backlog (proposed 2026-11-01)

Ordered by value to the user. Items already done are checked.

1. [x] **Notifications when someone needs you.** The Android app's background watcher does this (built, not yet tried on a real phone). Desktop notifications from DevRoom.exe are still to do.
2. [ ] **Use it away from home.** Reach the PC from mobile data through Tailscale (free) or a small relay.
3. [ ] **Reply from Dev Room.** Hooks cannot inject a message into a running session. The first step is a **Buksan sa Claude** button that opens that session in the Claude app.
4. [ ] **Cloud sessions** (claude.ai/code) in the office, next to the PC sessions.
5. [ ] **Daily report:** how long each agent worked, how often they waited for you, and context usage. It could be an end-of-day summary from the leader.
6. [ ] **One floor per project.** LOGIC_SIM, KWARTO_TV and others each get their own office, with an elevator or tabs to switch.
7. [ ] **Sound:** a soft ding when someone queues at your office, with an on/off switch.
8. [ ] **Day and night** following the real clock: the office darkens at night and the lamps turn on.
9. [ ] **Code signing** for DevRoom.exe so Windows SmartScreen stops warning. Needs a paid certificate, so it is optional.
