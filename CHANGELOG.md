# Changelog

Dev Room uses [semantic versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.
- **PATCH** for fixes.
- **MINOR** for new features.
- **MAJOR** for changes that break pairing, hooks or saved data.

The version lives in `VERSION`. Each build is published as release `v<VERSION>-build.<N>`; builds from `main` are marked Latest. Every CI run also has a build number, which the in-app updaters use to tell which build is newer. A version shows as `v0.3.0 (build 12)`.

## 0.10.0 (2026-11-01)

- **Playrooms you make.** "Piliin ang team" is now **Mga Playroom**:
  - Add as many playrooms as you like, name them, and pick **Office** or **Beach Resort** for each.
  - Put each session in its playroom with the number buttons, and mark the leader with ★.
  - Sessions not placed yet show in **Bagong dating**.
  - Saved on the PC (`team.rooms`, validated: at most 12 playrooms, 40 members each), so every device sees the same.
  - The phone's background alerts follow everyone placed in a playroom.
  - With no playrooms saved, the automatic units stay as before, and the editor starts from them.
- **Android updates inside the app.**
  - The new APK downloads into the app's cache, with progress shown, and Android's installer opens on it (FileProvider). No browser, nothing left in Downloads.
  - The first time, Android asks to allow installs from Dev Room. The app opens that setting and explains it.
- The playroom tabs scroll to the one you are in, and the title never pushes them off screen.
- The playroom editor scrolls as one page, and shows "7/9" so you can see how many fit.

## 0.9.0 (2026-11-01)

- **Updates arrive in seconds, not minutes.**
  - The PC checks for a new release every 30 seconds, instead of every 3 minutes.
  - A downloaded update installs within 2 seconds when the window is in the tray or minimized.
  - With the window open, it installs after 15 seconds without input (never while you are typing a message), after a 5-second countdown.
- **People walk around things.**
  - Walls and furniture are recorded on a half-tile grid while a scene is built.
  - Everyone (sessions, the crew and the staff) walks with A* around them and through the doors, then the path is straightened where the line of sight is clear.
  - Works in the office and the resort.
- **The PC app installs updates while minimized.**
  - Before, a minimized window waited for you forever. Only the UI updated; the exe (server features) did not.
  - Now a minimized window counts like the tray: the update installs within about 3 minutes and the window comes back minimized.
- **Units: several playrooms in one app.**
  - Unit 1 is your chosen team (or the biggest project). Every other project folder gets its own unit, shown as tabs under the title, with a red count when someone there needs you.
  - Units alternate between scenes:
    - **Office:** the power plant office.
    - **Beach Resort:** a co-working deck by the sea, nipa huts, a hammock grove, cabins, a beach bar, videoke, a pool and a sand volleyball court.
  - Same floor plan, people and pixel art. The resort has its own place names, activity words and staff (Bantay, Lifeguard, Housekeeping, Bartender).
  - A scene that repeats gets another colour.
  - The chosen unit is remembered on each device.
- **Free drag:** the playroom can be pulled well past its edges (for example out from under the panel).
- **Pictures and artifacts in the chat:**
  - Screenshots and images a session looks at (tool results), and pictures you paste, show inline.
  - Pages a session publishes (the Artifact tool) and files it sends (SendUserFile) show as cards, opened full screen inside the app.
  - Files are stored once by content hash under `%APPDATA%\DevRoom\att` (newest 300) and served at `/api/att/<id>`, with the pairing token required.
  - Pages and SVGs are served with a CSP sandbox and opened in a sandboxed iframe, so they cannot reach Dev Room or its token.
- **New composer, like the Claude app:**
  - One rounded box that grows with the text, with a round send button.
  - Recipient chips under **Para kay** replace the native dropdown.
  - The **Opsyon** (away mode) is gone, and a saved "on" or "auto" is switched off.
- The live transcript reader no longer drops the first reply of a session whose transcript was empty when first seen, and it saves what it reads, so a restart keeps replies and pictures.

- **Chat replies are formatted like in the Claude app:** bold, italics, inline code, code blocks, headings, bullet and numbered lists, and rules.
  - The text is escaped before formatting, so a reply can never inject HTML.
  - Links show as text.
- The project name in the chat header stays on one line, with an ellipsis.
- **Messages no longer get stuck:**
  - A message waiting for a busy session goes out as soon as that session is free, even when its turn ends without a Stop hook (an idle notification, or background work that stopped).
  - A turn stopped with Esc sends no Stop hook. A session with no hook and no transcript write for 10 minutes now counts as not working, so a message to it still goes out.
  - A queued message to a working session now says plainly that it arrives with the session's next step, like typing in the Claude app while it works.

## 0.8.0 (2026-11-01)

**Real-time chat in Dev Room's own chatbox.** The Chat tab is Dev Room's chat again, not claude.ai.
- **Live replies:** every session's transcript is followed as it is written, about once a second. Each reply (and each prompt you type in the Claude app) shows in the feed as soon as Claude writes it, not only when the turn ends.
  - Repeats from hooks and from the transcript are merged.
  - A session that is running when Dev Room starts is caught up from your last prompt.
- **Instant sending:**
  - A session that is not working is continued right away by Dev Room. It runs `claude -p --resume <session>`, with the message on stdin, in the session's folder and its permission mode (taken from the hooks).
  - A working session gets the message with its next tool call.
  - A message still waiting when a turn ends goes out right away.
  - If Claude Code cannot run (not installed, or not logged in), the message is marked "Hindi naipadala" and the reason is shown in the chat.
- Dev Room finds the Claude Code program on PATH, in `~/.local/bin`, in npm's folder, or in the Claude desktop app's copy. `DEVROOM_CLAUDE` overrides the choice.
- **Feed:**
  - A "•••" indicator shows each working session and what it is doing.
  - Your messages show once, with "Natanggap na niya ✓".
  - Errors appear as system notes.
- The Claude buttons and the claude.ai view on the Chat tab are gone. **Sagutin** opens that session's chat.

## 0.7.0 (2026-11-01)

**Malayo mode:** the phone reaches the office from anywhere (mobile data), with no extra app and no account.
- The PC app runs Cloudflare's free quick tunnel (`cloudflared`, downloaded once to `%APPDATA%\DevRoom\bin`). The tunnel points to a second, loopback-only port (4318).
  - Every request on that port counts as a phone, so the pairing token is always required (compared in constant time).
  - Hooks and the PC-only routes are refused on that port.
  - Host and Origin are checked against the tunnel's name.
- The tunnel's address changes on every start. The PC posts it to a private ntfy.sh topic named after a SHA-256 of the pairing token.
  - The phone app and its background alerts read it there when home Wi-Fi and Tailscale do not answer.
  - Only `*.trycloudflare.com` addresses are accepted.
- On by default, with an on/off switch and a live status in **I-connect ang phone**.
- The tunnel stops when Dev Room quits or restarts for an update.
- The phone header shows how it is connected: Wi-Fi, Tailscale, or "malayo, internet".
- No need to scan the QR code again.

## 0.6.6 (2026-11-01)

The Chat tab is the real Claude Code chat itself, on the PC and on the phone, so you read the sessions' latest replies and talk to them in real time without extra buttons.
- **Phone:** the Android app lays a native claude.ai view exactly over the office's Chat tab (like the PC's embedded view) instead of switching to a separate screen. It follows the tab's position, stays above the keyboard, keeps the sign-in, and Back goes back inside Claude first. Older office pages still use the full Claude screen.
- **One way in:** the separate **Claude** and **Kausapin sa Claude** buttons are gone wherever Claude is embedded; **Sagutin** on an alarm opens that session's Chat. A plain browser keeps the buttons.
- **Update status always on screen:** a pill in the header says plainly whether the PC app is up to date (with the last check time), downloading, installing or unable to check. On the phone it also shows whether the phone app itself is up to date, turns red with "pindutin para i-update" when it is not, and opens the download when tapped.
- **Claude screen:** respects edge-to-edge on new Android (status bar and keyboard), and Back uses the modern back handling.

## 0.6.5 (2026-11-01)

A full audit: a UI/UX inspection on PC, tablet and phone in light and dark mode, end-to-end tests of every flow against the real server, and a code review. Everything found is fixed here.

### Fixed: windows and the embedded Claude view (PC)
- The tray icon and a second launch bring the window back after the Claude view was opened.
- The Claude view only shows claude.ai, anthropic.com and the sign-in pages; other links open in your browser.
- The Claude view no longer gets stuck over other tabs. Hiding it gives the keyboard back to Dev Room.
- A minimized window is no longer restarted into the tray.

### Security
- **Web pages on the PC can no longer use Dev Room.** Any page on the PC, including one served on `localhost`, could read sessions and send them messages. Now:
  - every change needs Dev Room's own header;
  - hooks refuse browser requests;
  - only the Dev Room window can move the Claude view.
- **Android pairing:**
  - The Android app asks before a pairing link moves it to another PC.
  - The Claude screen never opens `devroom:` or other app links.

### Fixed: chat and alarms
- An acknowledged alarm now moves below the new ones.
- ACK is remembered after a reload.
- A new, different question from the same session raises the alarm again.
- **Toasts:**
  - With more than four toasts, the less urgent ones go first.
  - Tingnan closes its toast.
- **Messages:**
  - Two messages can no longer get the same id.
  - Text in angle brackets (`<user-card>`) is no longer removed from the feed; only Claude Code's system blocks are.
  - A failed message has **Subukan ulit** (try again) and **Alisin** (remove).
- Pause no longer makes the clock and timestamps run late.
- Clearing a session's name in the team picker restores the original name, and **Gamitin ang default** resets names too.

### Changed: UI/UX
- **One status vocabulary everywhere:** Nagtatrabaho, Kailangan ka, Naka-break, Pa-review, May error.
  - Permission prompts read in Tagalog and have a **Sagutin** button.
  - Acknowledged alarms look settled.
- **Phone:**
  - The tab bar never covers content.
  - The header is short, the chat list is tall, and one toast shows at a time.
  - Tapping a card filters the chat to that session.
- **Touch:** controls are at least 40 px on touch screens.
- **Chat panel:**
  - The Claude bar and composer are compact, and the away setting moved under **Opsyon**.
  - Toasts and the legend hide while the panel is open.
- **Dock:** it scrolls sideways with the mouse wheel and fades at the edge when more cards follow.
- **Keyboard:**
  - Dialogs take focus and keep Tab inside them.
  - The closed panel is out of the Tab order.
  - Cards keep focus.
- **Names:** shortened names end with "…", avatars use initials, and "Hooks" is a status, not a button.
- **Map:** 60 fps on 90 and 120 Hz screens; the Claude view follows a move to a monitor with other scaling.

### Fixed: server and Android
- Transcripts are read outside the shared lock.
- Bodies up to 32 MB are accepted, so a large Write is not dropped.
- `/api/team` refuses malformed data.
- An update no longer fails when an older copy is still in use.
- The background watcher closes its connections and starts fresh on a new pairing.

## 0.6.4 (2026-11-01)

### Changed
- **On the PC, the Chat tab is the real Claude Code chat.** claude.ai/code, with this PC's sessions through Remote Control, is shown inside the Dev Room window, over a wider panel. It no longer opens a separate window.
  - Sign-in windows (Google or email) open inside Dev Room in the same profile, so you stay signed in.
  - The **Claude** button opens this panel.

### Fixed
- **No more scrollbars at the bottom and right of the window.** The closed panel sat off screen and widened the page.
- **Red disk and trails on the map:**
  - A solid red disk appeared above the map. The alarm light piled up where the background was transparent, and other moving things could leave trails there. The background is now opaque.
- **Labels:**
  - Name tags no longer overlap; the one in front keeps its tag.
  - Bubbles stay inside the map.
- **Phone:**
  - The map controls fit in one row.
  - The alarm notice is compact.
  - The cards are no longer hidden by the tab bar.

## 0.6.3 (2026-11-01)

### Added
- **The official Claude Code chat inside Dev Room.** The **Claude** button on the map and **Kausapin sa Claude** in Chat open claude.ai/code, which lists this PC's sessions through Remote Control, for instant replies with no waiting.
  - **Android:** it opens on a screen inside the app. This needs the new app version; tap **I-update** on the banner.
  - **PC:** it opens in its own Dev Room window.
  - **Signing in:** you sign in to Claude once. If Google sign-in is refused inside the app, use the email code.

### Fixed
- **Sessions looked busy without replying.** Since 0.6.0, a finished session waited up to 20 minutes for a message from the app whenever a phone had Dev Room open. While it waited, the session stayed "running" in Claude Code and could not take prompts. This waiting is now off by default, and existing setups are switched to off once.
  - Messages from the app still reach a working session at its next step, and a stopped one with its next turn.
  - You can still turn the waiting on in Chat; it now warns what it does.
  - Press Esc in a session that is still waiting to end the wait.

## 0.6.2 (2026-11-01)

### Fixed
- **Chat on phones:**
  - While you type, the Office/Chat bar, the panel header, the tabs and the away setting step aside. The message box and the conversation stay in view above the keyboard. Before, the bar covered the message box.
  - The feed hides the system blocks that Claude Code adds to prompts, such as `<system-reminder>`.
  - A failed send says why (status and reason).
  - A message to a session that had already stopped before the app was opened explains that it arrives with that session's next turn.

## 0.6.1 (2026-11-01)

### Fixed
- **The map no longer blinks:**
  - Each piece of furniture is drawn exactly once per frame. Before, a piece was drawn a second time over the background whenever someone walked near it, which made glass walls and edges flicker.
  - The map resizes only at the start of a frame, and only when its size really changed. Before, a change in the status line or the update bar could flash a black frame.
  - The canvas no longer uses the low-latency (desynchronized) mode, which flickers on some GPUs.
  - The Android app keeps the PC address it is using and switches between Wi-Fi and Tailscale only after two failed checks in a row. Before, one slow reply reloaded the office.
- **Lighter on phones:**
  - The map is capped at 60 fps (120 Hz screens drew twice as often).
  - Cards, counts, the panel header and the status line are no longer rewritten every second, so taps on them are no longer lost.
  - A hidden page polls every 5 seconds instead of every second.
  - The background watcher downloads only changes, backs off when the PC is unreachable, tries the address that worked last time first, and stops after the pairing is revoked.
- **Chat:**
  - The feed no longer jumps to the bottom while you read.
  - A failed send stays in the feed and gives its text back to the input box.
  - The chosen recipient survives a team change; before, the message could go to another session.
  - The background watcher sends one alert per question, not one per update while a session waits, and no repeats after a restart.
- **Android app:** no more loop of watcher restarts, permission prompts and pairing-screen flips.
- **Phone layout:** the Chat tab shows only the chat, with the tab bar at the bottom.

### Security (from an audit)
- **A web page open on the PC can no longer drive Dev Room.** Before, such a page could read the pairing link, read conversations, send messages to sessions, or change hooks.
  - The server accepts only requests addressed to this PC by IP address or `localhost`, which blocks DNS rebinding.
  - Browser requests must come from Dev Room itself or the Android app.
- Artifact links must be web links, and the update banner's version text is cleaned.
- Session ids from hooks are checked before use.
- The number of Stop hooks that can wait for a phone message at once is capped.

### Fixed in the PC server (from an audit)
- **Messages are not lost when a hook is cancelled.**
  - A message counts as delivered (**Naipasa ✓**) only after the session's next hook confirms that Claude got it.
  - Otherwise it goes back to the queue after 3 minutes.
  - Messages no longer go to a subagent's tool call.
- **Waiting and restarts:**
  - A "naka-abang" (waiting) state can no longer stick after a restart or a cancelled hook.
  - The tray auto-restart waits while a session is waiting for a phone message.
- **Hooks and `settings.json`:**
  - Dev Room removes only its own hook commands. Before, it could also remove other local tools' hooks with `/hook` in the URL.
  - `settings.json` and Dev Room's own state are written atomically, so a crash can no longer unpair phones.
  - The hook command skips any HTTP proxy.
- **Server:**
  - One bad request can no longer stop the whole server.
  - Transcripts are checked without blocking other requests.
  - Sessions that ended without saying so are forgotten after 3 quiet days, even while Dev Room keeps running.
- **Second copy:** a second copy of Dev Room that cannot start its server now just opens the running one's window and exits.

### Not yet
- Updates are downloaded over HTTPS from this repository's releases but are not signed. Signing needs a key added to the repository secrets.

## 0.6.0 (2026-11-01)

### Added
- **Chat with your sessions from the app (PC, phone or tablet).** Nothing else opens.
  - **Choosing who gets it:** pick the session in **Para kay**, or tap a message or a filter chip.
  - **A working session** gets your message at its next step, as extra context for the turn it is on.
  - **A finished session** waits for your message and continues with it. By default it waits while a phone or tablet has Dev Room open (**Kapag may phone na nakabukas**). You can also choose **Palagi** (always) or **Hindi** (never). It waits up to 20 minutes; at the PC, press Esc in Claude Code to stop the wait.
  - **Delivery state:** each message shows **Naghihintay maipasa** (waiting to be handed over) or **Naipasa ✓** (handed over).
- **Team chat feed:** the prompts and replies of every session in one feed, with filter chips per session. Questions and waiting sessions show at the end.
- **Ano ang bago:** after each update, every device shows a one-time card with what is new and the steps to take (**Gawin mo**).
- **One Dev Room at a time:** starting `DevRoom.exe` again opens the window of the one that is running.

### Changed
- **Closing the window frees its memory and CPU.** Only the small server keeps running in the tray for phones, tablets and the hooks. The tray icon or **Buksan ang Dev Room** brings the window back.
- Dev Room now starts with Windows, hidden in the tray. You can turn this off in the tray menu.
- Phones and tablets start at a lighter resolution. The map stays at 60 fps.
- The Stop hook may now wait up to 20 minutes. Dev Room updates the hook by itself.

## 0.5.1 (2026-11-01)

### Changed
- **Updates install themselves on the PC.**
  - With Dev Room in the tray, it restarts into the new version right away and stays in the tray.
  - With the window open, it waits for a minute without input, then counts down 10 seconds. The countdown shows in fullscreen too, with **Ngayon na** and **Mamaya** (wait 20 minutes) buttons.
  - Phones and tablets reconnect by themselves after a few seconds.

## 0.5.0 (2026-11-01)

### Added
- **Off-duty shift crew:** Bea, Mika, Trish and Joy, each from a different shift. Hunger, sleepiness, the need to move, boredom and the CR build up over time and with the clock, and each of them picks what to do next from those needs plus her own favorites. So the office always has someone eating, sleeping in the tulugan, on the treadmill or lifting, or shooting hoops, without a fixed loop. They also join a chess, billiards or basketball game when someone is waiting for a partner.
- **Work you can see, by kind of work:**
  - Running a build or tests: the character stands behind the chair and watches.
  - Coding: typing, with a short stretch at the desk now and then.
  - Reading or researching: leaning in.
  - Each kind of work has its own lines.
- **Called back from a break:** a session that gets a new task says so ("Pause muna ang laro, may utos.") and hurries back to its PC.
- **New hooks:** PostToolUse and SubagentStop. Dev Room adds them to existing setups by itself; it keeps your other hooks.
- **Fullscreen shows a one-line strip of the session cards** under the map.

### Changed
- **New character art:** bigger 24 × 32 sprites with hair highlights and bangs, eye shine, blush, collar and cloth shading, hands and shoes.
- Sessions now take real breaks (gym, basketball, eating, napping) after a few idle minutes. The same activity does not come up twice in a row.
- Your character is labelled **Boss**.

### Fixed
- **Lag:** a frame takes about 2 ms instead of about 12 ms.
  - Furniture is drawn once into the background, and a piece is drawn again only when someone walks in front of it.
  - The map canvas is opaque.
  - Name tags and bubbles move with transforms.
  - A slow device lowers the map resolution by itself.
- **An active session sitting at chess or coffee.** A session can keep working after its turn ends: background agents or tasks, or a turn the session resumes on its own. Dev Room now treats any new writing to the session's transcript, or to its subagents' transcripts, as work, and goes back to idle once the writing stops for a minute.
- **Permission alarms clear as soon as the tool runs** after you approve it.

## 0.4.0 (2026-11-01)

### Added
- **The Dev Room is now a power plant**, still in isometric pixel art. The map is 42 × 23 tiles:
  - **Top row:** Control Room with a video wall (MW output, trend, alarm beacon) and console rows, Team Supervisor office for the Lead, your office, Library, and sleeping quarters.
  - **Bottom row:** Gate, CR and lockers, Pantry, Lounge, Rec Room, Gym, and a basketball half court.
- **Alarms tab with ACK:** a permission prompt or a trip is HIGH, a question is MEDIUM. The alarm blinks until it is acknowledged, on the console lamp and on the beacon.
- **Characters act out the real tool in use:**
  - **Typing code:** the screen fills with code.
  - **Reading or searching:** a scrolling page.
  - **Running tests or builds:** a progress bar, with a green or red flash for pass or fail.
  - **Web:** a globe on the screen.
  - **Planning:** a checklist.
  - **Delegating:** a small Subagent courier runs to the library and back with a report.
- **New NPCs, each with a routine:**
  - The janitor mops, cleans the CR and empties the bins.
  - Pantry staff brew coffee, serve and wipe tables.
  - Maintenance does PM on consoles, the video wall and the treadmill.
  - The guard patrols with a flashlight and signs the logbook.
  - The Safety officer does inspections and fatigue checks.
- **Real-time routines:**
  - Morning coffee, lunch from 11:30 to 1:00 (you included), merienda at 3:00, gym and basketball after 5:00, and a dimmer plant at night.
  - Activities take real-life minutes.
  - One thing leads to the next: coffee then CR, gym then CR and pantry.
  - Tired people pick coffee or sleep.
  - Nobody waits forever for a chess or billiards partner.
- **Status icons, effects, energy and mood, a timeline, and walking that faces the direction of travel** (from the 0.4 preview).
- **Pinch to zoom** on phones and touch screens.
- **Away from home:** when Tailscale runs on the PC, the pairing QR also carries the Tailscale address. The Android app and its background watcher use home Wi-Fi first and Tailscale when away.

- **Tray app:** closing the Dev Room window keeps it running in the system tray, so phones and tablets stay connected. The tray menu has Buksan ang Dev Room, Simulan kasabay ng Windows (start with Windows, hidden in the tray), and Isara nang tuluyan.
- **Widescreen map with a slide-over panel:** the Alarms, Chat and Detalye panel opens over the right side of the map only when needed. It opens from a character, a session card, an alarm, the alarm count or the Panel button, and closes with ×, Esc or a tap on the map. The session cards stay at the bottom.
- **Notifications in the lower right:** alarms stack up with ACK and Tingnan buttons and stay until handled. They move beside the panel when it is open.
- **Fullscreen is map only:** the header, status bar and dock hide, and the whole plant fits the screen. It also works where the browser blocks real fullscreen, such as the Android app.

### Fixed
- **Room names are painted flat on the floor**, following the isometric angle, instead of floating horizontally.
- **Bubbles never overlap each other or other people's name tags.** Colliding bubbles lift up with a leader line, and fewer people talk at once when zoomed out. Alarms and the selected character always get through.
- **Brighter floors.**
- **The leader is detected automatically** when no team is saved.

## 0.3.0 (2026-11-01)

### Added
- **Android app (APK):** shows the office from the PC and pairs through the QR code ("Buksan sa Dev Room app").
- **Notifications:** a background watcher alerts you when a session is waiting for you, even with the app closed.
- **App updates:** the Android app offers each new APK from the release.
- **Versioning:** a version number in both apps, a changelog, and a separate GitHub release per version.

## 0.2.0 (2026-11-01)

### Added
- **Office life:**
  - Every session is a woman with her own hairstyle and outfit.
  - You appear as the Boss.
  - The leader is detected automatically and stays at the LEAD desk, walking the floor to check on the team after a long idle.
  - Agents stay at their desks for a few minutes after a task and come back to check for work.
  - Break activities: TV, billiards, chess, darts, coffee, eating, stretching, phone and naps.
  - The guard plays ML on her phone and dozes off.
- **Bubbles:** they last longer, the lines are varied and include the real task, and A− and A+ set the text size.
- **Auto update:** the UI updates itself, and the exe swaps itself in with a one-click restart.

## 0.1.0 (2026-10-31)

### Added
- **Portable Windows app** (`DevRoom.exe`), fed in real time by Claude Code hooks, with one-click hooks setup.
- **Team picker** with renaming, and phone access over Wi-Fi through a QR code and pairing token.
