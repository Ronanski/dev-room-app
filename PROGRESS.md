# Dev Room: progress notes

An app that shows Claude Code sessions (main session + subsessions) as characters in a café-style office.
Working sessions sit at their PC. Idle ones take breaks. Sessions that need input line up outside the user's office.

## Status

Mockup only. There is no real app yet.

- Mockup: `mockup/dev-room-mockup.html` (single file, runs on sample data)
- Published preview: https://claude.ai/artifact/RcjPyRzi2NLL72JgatPTUw

## Decided

- **Look:** coffee-shop office with muted coffee, cream and earth tones. The only bright color is red, used for "needs you".
- **Rooms:** work floor, Lead's office, the user's office ("Opisina mo"), hallway with water cooler, lounge, CR, lobby, HR and coffee bar.
- **Characters:**
  - Main session = Lead, in its own office.
  - Subsessions sit at desks.
  - The user appears as "Ikaw".
  - NPCs are the Guard, who greets new sessions, and HR, who sends people on break and reminds the user who is waiting.
- **What each status looks like:**
  - Working: sitting at the PC.
  - Needs input: standing in line outside the user's office with a hand raised.
  - Idle under 10 min: coffee, CR or walking around.
  - Idle 10 to 30 min: chatting or eating.
  - Idle over 30 min: asleep in the lounge.
  - Done or error: an indicator at the desk.
- **Layout:**
  - Map on the left and a chat panel on the right with Chat, Detalye and Activity tabs and a message box.
  - Session dock along the bottom.
  - Event feed in the corner of the map.
- **Text:** full sentences only, with no truncated text or "…".
- **Platforms:** portable desktop app and Android app.
- **Session sources:** both local CLI and cloud (claude.ai/code).
- **Data source plan:** Claude Code hooks (SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop, SessionEnd) POST events to the app's server.

## Open questions

1. Should the view be isometric like the user's reference screenshot, or stay top-down pixel art?
2. What is the real session-name format? Roles are parsed from it, and the mockup assumes `[ROLE] task`, e.g. `[BE] Payments API refactor`.
3. Desktop shell (Electron or Tauri) and Android approach (Capacitor, React Native, etc.) are not decided yet.
4. Where should the server run so that both local and cloud sessions can reach it?
