# UI build scripts (0.4.0)

These scripts turned the 0.3.0 UI (`app/ui/index.html` on `main` at v0.3.0) into the 0.4.0 plant UI. They also build the sample-data mockup that was reviewed as an artifact.

`build.sh` runs them in order:

1. `mock04.py`: status icons, effects, energy and mood, timeline and walking (plus the mockup's sample team).
2. `cr_iso2.py`: the power plant map, the Alarms tab and ACK, and the plant wording.
3. `realism.py`: time-of-day routines, real durations, activity chains and night lighting.
4. `npcs.py`: brighter floors, pinch zoom, new NPCs, and characters that act out the tool in use.
5. `bubbles.py`: non-overlapping bubbles and the talk budget.
6. `unmock.py`: back to the real data source, for the app.
7. `tsui.py`: Tailscale hints in the pairing dialog.

From 0.4.0 on, `app/ui/index.html` is the source of truth and is edited directly. The scripts are kept as a record of the change. They only apply to the 0.3.0 file.
