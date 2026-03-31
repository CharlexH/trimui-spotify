# SideB FAV LIST Navigation Design

**Date:** 2026-03-31

**Goal:** Improve `FAV LIST` navigation so selection wraps around at the ends and supports playlist-only long-press repeat with a 300 ms hold delay and 120 ms repeat interval.

## Scope

- Make `PlaylistUp` wrap from the first item to the last item.
- Make `PlaylistDown` wrap from the last item to the first item.
- Add long-press repeat only while the playlist overlay is visible.
- First press moves immediately; repeat begins after 300 ms; subsequent repeat interval is 120 ms.

## Constraints

- The change applies only to the playlist overlay, not normal playback controls.
- Existing button semantics for `A/B/X/Y` must remain unchanged.
- Repeat must stop immediately when the D-pad is released or the playlist overlay is dismissed.

## Chosen Approach

Use two small helpers:

- a pure selection helper in the command layer to compute wrap-around movement
- a pure repeat-state helper in the input layer to manage hold delay and repeat timing

At runtime, the input thread updates repeat state from `ABS_HAT0Y` events while a lightweight repeat loop emits `PlaylistUp` or `PlaylistDown` commands only when due and only when the playlist overlay is visible.

## Testing

- Add unit tests for wrap-around selection behavior.
- Add unit tests for repeat timing:
  - first move is immediate
  - repeat does not start before 300 ms
  - repeat starts at 300 ms and then repeats every 120 ms
  - releasing the axis stops repeat
