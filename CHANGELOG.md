# Changelog

Dev Room uses [semantic versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.
- **PATCH** for fixes.
- **MINOR** for new features.
- **MAJOR** for changes that break pairing, hooks or saved data.

The version lives in `VERSION`. Each build is published as release `v<VERSION>-build.<N>`; builds from `main` are marked Latest. Every CI run also has a build number, which the in-app updaters use to tell which build is newer. A version shows as `v0.3.0 (build 12)`.

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
