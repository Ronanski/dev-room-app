# Changelog

Dev Room uses [semantic versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.
- **PATCH** for fixes.
- **MINOR** for new features.
- **MAJOR** for changes that break pairing, hooks or saved data.

The version lives in `VERSION`. Every CI run also has a build number, which the in-app updaters use to tell which build is newer. A version shows as `v0.3.0 (build 12)`.

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
