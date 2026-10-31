# Dev Room: progress notes

## The idea

The user runs Claude Code projects with one **main session** (the leader) and several **subsessions**. They work like branches, and each one has its own role.
Today the user monitors them one by one.

Dev Room shows them all in one place instead. Each session is a character in a shared office, so the user can see at a glance:

- who is working,
- who needs the user's input,
- what each session is doing right now, and how far along it is.

## Status

Mockup only. There is no real app yet.

- Mockup: `mockup/dev-room-mockup.html` (single file, runs on sample data)
- Published preview: https://claude.ai/artifact/RcjPyRzi2NLL72JgatPTUw

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
- [ ] Each session's role comes from its **session name**, which follows a fixed format. The real format is still to be provided; the mockup assumes `[ROLE] task`.
- [ ] Data source plan: Claude Code hooks POST events to the app's server.
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

1. What is the real session-name format? We need 2–3 examples.
2. Which stack should the build use?
   - Desktop: Electron or Tauri.
   - Android: Capacitor or React Native.
3. Where should the event server run so that both local and cloud sessions can reach it?
