# Changelog

Dev Room uses [semantic versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.
- **PATCH** for fixes.
- **MINOR** for new features.
- **MAJOR** for changes that break pairing, hooks or saved data.

The version lives in `VERSION`. Each build is published as release `v<VERSION>-build.<N>`; builds from `main` are marked Latest. Every CI run also has a build number, which the in-app updaters use to tell which build is newer. A version shows as `v0.3.0 (build 12)`.

## 0.18.3 (2026-11-03)

- **Hooks in every Claude folder on the PC.** The user switched to a Claude account with Claude Projects; if that account keeps its own folder (`CLAUDE_CONFIG_DIR`, e.g. `~/.claude-work`), its threads sent no hooks at all, because Dev Room installed them only in `~/.claude/settings.json`. Without hooks there is no live status, no chat and no `session_title`, so every earlier name fix could not reach those threads. Once the hooks are set up in `~/.claude` (your consent), the same hooks now go into every other Claude folder the scan finds (`install_hooks_everywhere`, each minute), with the same merge and backup, keeping the folder's own hooks and settings.
- **Detalye shows the last hook** from the session: which event, when, and whether it carried `session_title` (and its value), or that no hook came at all (found only by the scan). `/api/state` records carry `last_hook` and `last_hook_at`.
- Tested here (scratch server build): with hooks set up in `~/.claude`, a second folder `~/.claude-work` with its own `PreToolUse` hook and `model` gets all eight Dev Room hooks, a backup, and keeps its own hook and setting; `last_hook` shows `UserPromptSubmit · session_title = "SANA : PIXEL OFFICE"`. Not tested: the real PC.

## 0.18.2 (2026-11-02)

- **Thread names from the Claude app and Claude Projects, for real.** The user's screenshots showed names such as `SANA : PIXEL OFFICE` and `GISELLE : KATULONG NG FIXER` in the Claude app's project overview while Dev Room showed folder names. Claude Code 2.1.3xx sends the session's current name as `session_title` in its hook input (SessionStart, UserPromptSubmit), including renames made in the Claude app or by a project's coordinator (`rename_session` from the host). With its newer storage that name may never reach the transcript file, which is why reading the transcript and the sidecar was not enough. `server.rs` now keeps it as `hook_title`, which comes first after a Dev Room rename; Detalye shows "Pangalan sa Claude app (galing sa hook)".
- A thread that is already running gets its name with its next hook (the next message from you or the coordinator).
- Tested here (scratch server build): no `session_title` keeps the old behaviour; SessionStart with it shows the name; a later UserPromptSubmit with a new name renames the session; a Stop hook without it keeps the name. Not tested: the real PC.

## 0.18.1 (2026-11-02)

- **The phone's tabs stay clear of the navigation keys even before the new phone app is installed.** An Android app older than 0.18.0 sends no bar sizes; the office (inside the app, on Android) now keeps 28 px at the top and 48 px at the bottom until real sizes arrive, instead of 0.
- **Detalye shows where a session's name came from** (Dev Room rename, Claude rename, Claude's own title, your first prompt, the coordinator, or the folder because the transcript has no name), whether a `custom-title.json` exists, whether it was found by hooks or by the scan, and its transcript path. `/api/state` records carry `title_src`, `transcript`, `sidecar` and `found_by`. The user reported that renames and the phone fixes did not work on the real devices; this makes the cause visible there instead of guessed.
- Tested here: a scratch server shows the right source for a sidecar rename, a transcript rename and a first prompt; inside an app-like frame without bar sizes the tabs end 48 px above the bottom; no page errors. Not tested: the real phone and PC.

## 0.18.0 (2026-11-02)

- **Renames show up again.** Claude Code 2.1.3xx can keep a rename only in a sidecar file, `<transcript folder>/<session id>/custom-title.json` (`{"customTitle": …}`), for renames from the Claude app or a Claude Projects thread. Dev Room never read it, so renamed threads kept their old names. `server.rs` now reads it when a session is first seen (it wins when it is newer than the transcript's last write) and watches it live in the transcript reader (re-read only when the file changes), after the transcript's own lines so an old name in a session's history never covers it.
- **Every session on the PC is detected, also without hooks.** Each minute the PC reads every Claude folder (`~/.claude`, `CLAUDE_CONFIG_DIR` and other `~/.claude-*` folders, one per account) and adds sessions whose transcript changed in the last 48 hours, with their name, folder and last answer; the liveness check then shows them working while their transcript is written. Subagent transcripts and Dev Room's own quick-answer copies are skipped, and a session that really ended (SessionEnd) is remembered in `gone` so it never comes back.
- **Mga Playroom:** **I-scan ang PC (huling 14 araw)** finds older sessions too (kept a day so they can be placed); the list is grouped by project with **Ilagay lahat sa** buttons that put a whole project in a playroom.
- **Phone: nothing hides under the status bar or the navigation keys.** The Android app draws edge to edge, and the office runs in an iframe where `env(safe-area-inset-*)` is always 0, so the Office/Chat tabs sat under the navigation keys and the header under the status bar. The app now measures how much of its web view the bars cover (`DevRoom.insets`, from where the web view really is) and passes it to the office (`devroom:insets`), which uses `--sat`/`--sab`/`--sal`/`--sar` everywhere it used `env()`.
- **Phone Lite is exactly one screen high:** the tabs sit at the bottom in the flow, and the list or the chat fills the space between, so the chat never runs past the screen.
- **The typing layout ends when the keyboard closes,** also when it closes with Android's back key or a gesture while the message box keeps the focus (the tabs and the chat header stayed hidden, so you could not get back).
- **Android's back key:** from a chat back to the list, closes a panel, the viewer or the keyboard; on the list it leaves the app (minimize) instead of doing nothing.
- **Lite: the office's height is adjustable** on the phone and the tablet. Drag the bar under it (remembered in `devroom.liteh`; double tap resets); a smaller office gives the list and the chat more room.
- Tested here (a scratch build of `server.rs`): sidecar renames on start, live, after Stop hooks, and a newer `/rename` still wins; the scan finds sessions in two account folders, skips old, subagent and quick-answer transcripts, adds a 10-day-old one on I-scan, and never brings back an ended session; Playwright at 1366×768 and 800×1280 (grouping, scan, Ilagay lahat, save) and at 800×1280 and 390×844 (height bar, remembered, tap on a person, reset); the 15 Lite checks still pass; no page errors. Not tested: the real PC with its own Claude folders. Phone checks in an app-like frame (Android 13 WebView user agent, 390×844, a 30 px status bar and 48 px navigation keys): tabs above the keys, header below the status bar, one screen high, chat box above the tabs, typing layout ends after the keyboard closes, back key steps; the bar measurement itself (Java) is compiled by CI only. This one has a phone APK: with 0.17.0 installed it installs by itself.

## 0.17.0 (2026-11-02)

- **The phone app updates itself without a tap (Android 12 and newer).** The update downloads quietly and installs through an install session with `USER_ACTION_NOT_REQUIRED`, which Android allows for an app updating itself. It installs when you leave the app; if the app stays on screen (a tablet showing the office), a 15 s countdown with **Ngayon** and **Mamaya** (Mamaya waits an hour or until you leave the app). The app closes for a moment; open it again. On Android 14+ Dev Room also asks to own its updates. If Android still wants a confirmation, it opens it, and a notification "I-tap para i-install" is posted. Older Android keeps the I-update banner and Android's Install button.
- **No APK when the Android app did not change.** CI stamps the APK and `version.json` with `shell`, the git tree id of `mobile/`. The phone downloads a new APK only when that id differs; releases that change only the office (served by the PC) download nothing. A `version.json` without `shell` still updates by build number.
- **Alerts come back by themselves after an update:** `UpdateReceiver` starts the background watcher again on `MY_PACKAGE_REPLACED`.
- The office header says "Phone app: kusang nag-a-update" while this happens. A failed install is shown the next time the app opens, with **I-install** (the manual way).
- **Once more by hand:** the APK that brings this (0.17.0) still installs the old way: press I-update, then Install.
- Tested here: the phone shell's update flow in Playwright with a stand-in plugin and server (same shell: nothing downloaded; new shell: quiet download, countdown, Ngayon installs silently, Mamaya waits, leaving the app installs at once, older Android gets the banner, an old `version.json` still updates). The Java (install session, receiver) is compiled by CI only. Not tested: a real Android phone or tablet.

## 0.16.2 (2026-11-02)

- **Messages a Claude Projects coordinator passes to a thread are readable.** They arrive as the thread's prompt, wrapped in `<project_claude_message>` with `<relay>`, `<cited>` and `<note>` (an underscore tag, so the system-prompt filter missed it), and the chat showed the raw tags. The chat now shows them as **Coordinator → thread**, with your cited words under your name and the coordinator's note (`relayMsg` in the UI; `relay_text` in `server.rs`). The session's detail reads "Utos ng coordinator: …".
- **Thread names when Claude Code wrote none.** A thread in its own folder is named after the folder, as words: `winter-test` is Winter Test and `code-fixer-b7c2` is Code Fixer (the hash is dropped, and the role is detected from the words). Threads that share a folder (`icms-master-tool`) are named after the coordinator's first message to them (`relay_title`, used only when the folder is shared). The Lite list shows the whole name.
- **Tablet Lite: the chat's width is adjustable.** Drag the bar between the list and the chat (remembered in `devroom.litecol`; double tap resets).
- In Lite, the chat header shows the real status (Tapos or Naka-break), never the big map's pastimes such as "Nagse-cellphone". "Tapos na ang agent (agent)" now reads "Tapos na ang isang agent".
- Tested here (a scratch build of `server.rs` fed with hook JSON and coordinator relay prompts like the ones on the user's tablet; Playwright with the Android 13 WebView user agent at 800×1280 dark and 390×844): no raw tags in the chat, your words and the note shown, names Team Files, Winter Test, Code Fixer and the coordinator's task for the shared folder, the bar widens the chat, is remembered after a reload and resets on double tap, no bar on the phone, no page errors; the 15 Lite checks still pass. Not tested: the real PC and tablet. A thread that started before this version and gets no new coordinator message keeps the folder name.

## 0.16.1 (2026-11-02)

- **Thread names from Claude Projects show up.** Claude Code 2.1.3xx names a session with an `agent-name` line (`agentName`) in its transcript. A project's coordinator names its threads only that way, and `/rename` writes it next to `custom-title`. Dev Room read only `custom-title`, `ai-title` and `summary`, so a thread showed Claude's own title or, before that, the folder name (every thread in one folder looked the same). `title_of` in `server.rs` now reads `agent-name` as the name you gave, in the tail read, the first full scan and the live reader.
- **Threads with the same role are told apart by name.** Three threads named `FIXER : NINGNING`, `FIXER : GISELLE` and `FIXER : WINTER` showed as Fixer, Fixer 2 and Fixer 3. Now the tiles, the map and the chat say Ningning, Giselle and Winter, and the role reads `Code Fixer · NINGNING`. A number is used only when there is no name after the colon. The Lite list shows the role and the name (`Fixer · NINGNING`), and the leader's full name.
- Tested here (a scratch build of `server.rs` fed with hook JSON and transcripts): a thread with only `agent-name` shows that name; `custom-title` and `ai-title` still work; an `agent-name` written while the session runs renames it live; three FIXER threads show as Ningning, Giselle and Winter on the PC tiles and map and in the Lite list (Playwright, 1366×768 and 390×844, no page errors; the 15 Lite checks from 0.16.0 still pass). The old server showed `Fix parser tests` and `icms-master-tool` for the same transcripts. Not tested: the real PC and a real Claude Projects thread.

## 0.16.0 (2026-11-02)

- **Lite view for phone and tablet (portrait),** from `docs/lite-mockup.html`. It is on by default on a touch screen held upright; landscape and the PC keep the full view.
  - **A small live office on top:** people sit at their desks while they work, stand at the door with a raised hand when they need you, walk around when they are done, and rest on the sofa after 30 idle minutes. Tapping a person opens her chat. Only this small canvas is drawn (about 30 fps), so the big map no longer runs on the phone, which saves battery.
  - **An alert bar** for the first session that needs you (or failed), with **Sagutin**.
  - **Filters:** Lahat, Kailangan ka, Gumagana and Tapos, with counts.
  - **The session list:** face, name (★ for the leader), the real status (never the big map's pastimes, like billiards), what she is doing or the question, and the progress bar with n/m. Sessions that need you come first.
  - **Phone:** the Office tab is the small office and the list; tapping a session opens the chat with a ‹ back arrow. **Tablet:** the office across the top, the list on the left and the chat on the right. While typing, only the chat stays.
  - **Buong view (malaking office)** at the end of the list switches this device back to the full view (remembered); **Lite view** in the header turns it on again.
- Tested here (Playwright with the Android WebView user agent, against a stand-in server with the same `/api/state`, `/api/convo` and `/api/send`): 360×740 and 390×844 phones, an 800×1280 tablet in dark mode, 1400×900 PC. Filters, Sagutin, sending a message, the back arrow, tapping a person, the alert clearing when a session goes back to work, the full-view switch (also after a reload), landscape and typing on the tablet all pass, with no page errors and no sideways scroll. Not tested: the real phone, the real tablet and the real PC server.

## 0.15.2 (2026-11-02)

- **The plan-limits meter is gone** (the **5 oras** and **Linggo** chips in the header, added in 0.13.0). It read the Claude sign-in token from `~/.claude/.credentials.json` and sent it to the usage service every 2 minutes. Using a Claude login token outside Claude Code breaks the Consumer Terms, so it could get the account banned. Removed: `fetch_usage`, `claude_token` and the usage thread in `server.rs`, `usage` in `/api/state`, and `renderUsage` / `#usage` in the UI. Dev Room never reads the sign-in file now.
- Tested here (Windows PC, a scratch build of the server with Playwright): the header has no limits chip at 1366×768, 1920×1080, 800×1280 and 412×900, light and dark, with no page errors; `/api/state` has no `usage`. Also re-tested six 0.14.3 fixes that were never checked: tapping a character on a phone (also inside an iframe like the Android app), the colour around the map (office and resort, light and dark), the pairing card scroll, Sagutin putting the cursor in the message box, Mga Playroom on touch (no keyboard, Save bar in view) and safe areas (emulated Android bars). All pass. Not tested: the real phone and tablet.

## 0.15.1 (2026-11-02)

- **The PC app updates itself again.** After the exe swap the running image is renamed to `DevRoom.old-<build>.exe`, and the current-exe path then points at that renamed file. The restart started "the current exe", which was the old version again, so the PC stayed on v0.11.0 (re-downloading every check) until the user closed it and opened `DevRoom.exe` by hand. The updater now fixes the install path once at start (`app_exe`: always `DevRoom.exe`, also when running from an old copy) and uses it for the swap, the restart and the start-with-Windows entry (rewritten at each start).
- **Slow lines no longer fail the download:** a read timeout (60 s without data) replaces the 60 s limit on the whole 8 MB download.
- **An update that cannot be installed says why** in the header (the error was hidden behind "Dina-download").
- Tested here: a Linux build that renames itself shows the same path change, and `app_exe` stays `DevRoom.exe`. Not tested on Windows. The copy now running on the PC (v0.11.0) still has the old updater, so it needs one last manual close and reopen.

## 0.15.0 (2026-11-02)

- **Session names come from Claude Code again.** Current Claude Code writes its own title as `ai-title` (`aiTitle`) and a rename as `custom-title`; Dev Room only knew `summary`, so it fell back to the first prompt. It now keeps both (`custom_title`, `auto_title`): a Dev Room rename first, then your `/rename`, then Claude Code's title, then the first prompt. The live transcript reader picks up a new name within a second, so a rename in Claude Code or the Claude app shows at once (when it is written to the session's transcript on the PC).
- **A rename no longer reverts.** The Mga Playroom editor compared each name box with the shown title, which already was your rename, so any click in the editor deleted it. It now compares with the session's own name (`own_title` in `/api/state`).
- **Rename and move from the session's panel.** ✎ next to her name (Enter saves, Esc cancels, "Ibalik sa pangalan sa Claude" removes the rename) and a Playroom picker under it (with "+ Bagong playroom…"). Names on the map and tiles follow at once.
- The panel header no longer re-renders every second (unsaved units got a new id each time).
- Tested here with transcripts holding `custom-title` and `ai-title` lines, a live rename appended to a transcript, and renames and moves at 1366×768 and 412×900. Not tested: the real Claude app rename on the PC.

## 0.14.3 (2026-11-02)

A full test pass of every button and flow on PC (1366, 1920, touch 1280), tablet (portrait and landscape, with the keyboard) and phone (portrait and landscape, Android WebView user agent, inside an iframe like the shell), plus every hook event and API route. 22 bugs fixed:

**Server and chat**
- **An auto-compact no longer makes a working session look done.** `SessionStart` with `source: "compact"` comes in the middle of a turn; it used to set the session idle, so a message from you started a second run of it. Now it only updates the detail.
- **A project folder named like "subagents" no longer hides the session.** Only a transcript inside a `/subagents/` folder marks a subagent (`is_sub_path`).
- **Leaving a session (`/exit`, `/clear`, logout) right after chatting removes it after a minute.** Only a plain end (`reason: "other"`) can be Dev Room's own run; liveness waits only while a run of ours is going.
- **A late ⚡ answer is dropped** when the session itself already answered.
- **Every waiting message gets a ⚡ answer.** Messages that arrive while one is being answered are answered together next.
- **The same text said twice stays twice** (two "continue" prompts). The hooks and the transcript reader now pair copies by source instead of hiding any repeat within 30 minutes.
- **Saving the team without rooms keeps your playrooms** (`/api/team` keeps parts left out).
- **Queue reason "ask"** when the session waits on a permission prompt; an ended session no longer stays "Kailangan ka".

**Tablet and phone**
- Tablet portrait with the keyboard open: only the chat shows, so the message box stays in view.
- Phone landscape (short screens): the page scrolls, the office keeps its size and the panel takes the screen.
- The phone's Office/Chat bar sticks to the bottom again (`overflow-x:clip` instead of `hidden`).
- Tapping a character on a phone opens her chat.
- Safe areas (Android status and navigation bars) are respected.
- Mga Playroom on touch no longer pops up the keyboard; its Save bar stays in view.
- Bigger zoom and name inputs for fingers; no double "…" in the placeholder.

**PC**
- "Buong office" and fullscreen show the whole office, centred.
- Markdown tables render in the chat; one-line summaries drop markdown symbols.
- The selected "Para kay" chip is scrolled into view; the recipient and unit rows scroll with the mouse wheel.
- Switching panel tabs starts at the top (Detalye) or at the newest message (Chat).
- The area around the map matches the scene's colour (no seam).
- Names with Ñ and other letters are capitalised correctly.
- The pairing card scrolls; "Sagutin" puts the cursor in the message box.

## 0.14.2 (2026-11-02)

- **A session you chat with no longer disappears.** The SessionEnd hook of Dev Room's own run (`claude -p --resume`, which hands your message over) can arrive after the run has exited; Dev Room had already forgotten the run, took it for a real end, and removed the session a minute later. Now any SessionEnd within two minutes after Dev Room's own run of that session is ignored (`ran_recently`), and a session you placed in a playroom is removed only after ten quiet minutes after a real end.
- **Quick-answer copies never show up as sessions.** If the copy's hooks fire despite `disableAllHooks`, its prompt (marked `[Dev Room] Mensahe ni Boss habang…`) makes Dev Room drop and ignore that copy.
- Tested with a stand-in `claude` whose SessionEnd arrives two seconds after it exits, and whose quick-answer copy fires every hook: after 75 seconds both chatted sessions were still there and no copy appeared.

## 0.14.1 (2026-11-02)

- **Typing on a portrait tablet no longer squeezes the app into a narrow strip.** The class the app set while you type (`typing`) was also the chat's "working…" line class, which centers its content; on the whole app it shrank everything to the width of its content. It is now `kbopen`. Tested: with the keyboard open (800×620 on a portrait tablet) the page stays 800 px wide.
- **Portrait stays portrait while the keyboard is open.** The keyboard leaves a short, wide space, which used to flip the tablet to the landscape layout. Portrait now comes from the device's orientation (`screen.orientation`, class `port`), and the map keeps its zoom while you type.
- **Letters no longer come out reversed.** The message box's placeholder was set again on every refresh while you typed, which broke the phone keyboard's input; it is now set only when it changes, and the box's height only when it changes.
- **A session no longer disappears after you chat with it.** The run that hands your message over (`claude -p --resume`) ends with a SessionEnd hook while the session is still open in Claude Code, and Dev Room removed the session. SessionEnd from Dev Room's own run is now ignored; a real end removes the session after a quiet minute (a newer hook cancels it). Tested with a stand-in `claude` that sends SessionEnd: the session stayed; a real SessionEnd removed its session after a minute.

## 0.14.0 (2026-11-01)

- **Answers in real time, even while a session is busy.** Claude Code takes a new message only between steps. If yours is still waiting after 2.5 seconds (the session is in the middle of a long step, such as an agent or a test run), Dev Room asks a copy of that session right away (`quick_answer`: `claude -p --resume <id> --fork-session --permission-mode plan --settings {"disableAllHooks":true}`):
  - The copy has all of the session's context, answers in read-only plan mode, and fires no hooks (no extra character, no changes).
  - Its answer shows in the chat as **⚡ Agarang sagot habang abala**; your message says "Sumasagot na siya ngayon…" and then that it was answered.
  - The message stays queued for the session itself, which acts on it after its current step (PostToolUse, as before).
  - Tested with a stand-in `claude`: the answer arrived about 6 seconds after sending (with the old 4-second wait), no extra session appeared, and the next PostToolUse still handed the message to the session.
- **Fullscreen in the Android app no longer squeezes the page into a narrow strip.** Inside the app (an iframe in its WebView) Dev Room no longer calls the browser's Fullscreen API, which the app's WebView does not support; fullscreen view fills the screen with CSS. Anywhere, a fullscreen that leaves the page narrower than the window is undone while the view stays in fullscreen mode.

## 0.13.0 (2026-11-01)

- **Progress you can see.** Dev Room follows each session's own task list (its `TodoWrite`, `TaskCreate` and `TaskUpdate` calls, `track_tasks`):
  - Tiles show a progress bar and `done/total`; the map shows a small bar under the session's name while she works.
  - **Detalye** starts with the phase (**Sinimulan**, **Ginagawa**, **Tapos na**), a progress bar, and the checklist (✓ done, ▶ the current one, ○ waiting).
  - A fresh list starts when you give new work after everything was done.
- **Life in the playroom after work.** When a session finishes a turn after real work (over a minute), she walks to your office with her report ("Tapos na po, Boss…"), the Boss answers, and she goes back to her desk.
- **Plan limits in the header.** The **5 oras** and **Linggo** meters show how much of your Claude limits is used and when each resets (amber from 70%, red from 90%). They are read on the PC with the sign-in Claude Code keeps there (`~/.claude/.credentials.json`); only the percentages and reset times are passed on, never the sign-in. If it cannot be read, the meter says why.
- **Why a message waits.** Under a queued message, the chat now says exactly why (`queue_why`): the session is in the middle of a step (which one, and for how long), it is still answering your previous message, or Dev Room cannot find Claude Code on the PC. Claude Code takes a new message only between steps, like the Claude app's queue.
- Finding Claude Code on Windows also asks Windows for the running `claude.exe` when it is not in the usual folders.
- Tested here with Playwright and a stand-in usage service at 1366×768, 800×1280 and 412×900: meters, tile counts, map bars, the checklist, the queue reason and the report walk; no page errors, no overflow. The real usage service and the PC's sign-in file were not tested here.

## 0.12.0 (2026-11-01)

- **Session tiles.** The cards under the office are now equal tiles that fill the full width: up to 4 a row (`dockCols`: fewer when the screen is narrow, 2 on a phone), and the rest wrap below instead of scrolling sideways.
  - Each tile shows the session's **face**, cut from her own pixel sprite (`faceURL`, cached), and **one name** (the short name). A dot on the face shows the status; PC and landscape also show the status pill.
  - Lower tiles in portrait and in fullscreen (one line, 42 px and 36 px), with no timeline strip.
  - The same faces are used for the other avatars (panel header and lists).
- **Adjustable panel height in portrait.** Drag the bar above the panel up or down (`#pSplit`); the height is remembered (`devroom.chath`), and a double tap resets it.
- Tested here with Playwright at 800×1280 (dark and light), 1280×800, 1366×768, 1920×1080 and 412×900: equal tile widths, 4 per row (2 on the phone), faces loaded, drag and remembered height, fullscreen, no page errors, no overflow.

## 0.11.1 (2026-11-01)

- **Messages reach a lead that has agents running in the background right away.** Subagent hooks carry the parent's session id; they used to mark the parent as working (and `SubagentStop` turned an idle parent into "working"), so a message to a lead whose own turn was over sat in the queue for up to ten minutes. Now subagent hooks (with `agent_id`, or a `subagents` transcript) only update the parent's detail text while it works, never its status or its transcript; `SubagentStop` never marks a free session busy. Tested with a fake `claude`: the message went out in under a second.
- **Claude Code's own turns are no longer shown as your messages.** Prompts that Claude Code writes itself (`<task-notification>`, `<side-sessions-event>` and similar wrappers, for example "None of the sessions you started is working now…") are left out of the chat, from the hooks, the transcript reader and messages already saved.
- **Tablet portrait layout.** With the panel open on a tall screen wider than a phone, the panel docks under the office (office, session cards, then the chat across the full width) instead of covering half of the office, in normal and fullscreen view. A tall office view zooms so the office fills it.
- The status line is cleared when a turn ends.

## 0.11.0 (2026-11-01)

- **The PC window comes back as you left it.** Dev Room remembers the window's place, size and whether it was maximized (`window.json` in the data folder, saved on move, resize, close and before an update restart).
  - After an update restart it returns in the same state: in the tray, minimized on the taskbar (shown with `SetWindowPlacement` and `SW_SHOWMINNOACTIVE`, so it does not flash or take focus, and it opens maximized again if it was), maximized, or at its old place and size.
  - Reopening it from the tray uses the same place and size. A place on a screen that is no longer connected is ignored (the window is centered).
- **Artifacts and pictures open in a pop-out, not full screen.** One layout per device:
  - **PC and tablet landscape:** a floating window beside the chat. Drag it by its bar, resize it from the corner; it remembers where you put it. The chat stays usable while it is open.
  - **Tablet portrait:** a centered card.
  - **Phone (and phone landscape):** a sheet from the bottom; swipe it down to close.
  - **⤢** (or a double click on the bar) makes it full screen and back. Pictures: tap to see them at full size, tap again to fit. Esc, ✕ or a tap outside closes it.
- Tested here with Playwright at 1366×768, 1920×1080, 1280×800 and 800×1280 touch, and 412×900 and 900×412 phone (light and dark): drag, resize, remembered place, full screen, image zoom, swipe to close, no page errors, no overflow. The window restore needs the real Windows build to check.

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
