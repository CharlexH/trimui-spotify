# FAV LIST Navigation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add wrap-around `FAV LIST` navigation and playlist-only long-press repeat with a 300 ms start delay and 120 ms repeat interval.

**Architecture:** Playlist selection remains command-driven through `InputAction::PlaylistUp` and `InputAction::PlaylistDown`, but the selection math is extracted into a small helper so wrap-around behavior is testable. Long-press repeat stays isolated to the playlist input path in `input.rs`, where a repeat-state helper and lightweight repeat loop emit movement commands only while the overlay is visible and the D-pad is held.

**Tech Stack:** Rust binary crate, input event handling, std threading and timing, unit tests in existing module test blocks.

---

### Task 1: Lock wrap-around selection behavior with failing tests

**Files:**
- Modify: `spotify-ui-rs/src/main.rs`
- Test: `spotify-ui-rs/src/main.rs`

**Step 1: Write the failing test**

Add tests for:
- moving up from index 0 wraps to the last item
- moving down from the last item wraps to index 0
- zero-count lists stay at index 0

**Step 2: Run test to verify it fails**

Run: `cargo test playlist_selection`
Expected: FAIL because the current selection logic clamps instead of wrapping

**Step 3: Write minimal implementation**

Add a pure helper for playlist selection movement and use it in the `PlaylistUp` and `PlaylistDown` command handlers.

**Step 4: Run test to verify it passes**

Run: `cargo test playlist_selection`
Expected: PASS

### Task 2: Lock playlist long-press timing behavior with failing tests

**Files:**
- Modify: `spotify-ui-rs/src/input.rs`
- Test: `spotify-ui-rs/src/input.rs`

**Step 1: Write the failing test**

Add tests for a repeat helper that prove:
- first directional press emits an immediate move
- no repeat occurs before 300 ms
- repeat begins at 300 ms
- subsequent repeats occur every 120 ms
- release clears the hold

**Step 2: Run test to verify it fails**

Run: `cargo test playlist_repeat`
Expected: FAIL because the helper does not exist yet

**Step 3: Write minimal implementation**

Implement the repeat-state helper and the runtime repeat loop used only by playlist navigation.

**Step 4: Run test to verify it passes**

Run: `cargo test playlist_repeat`
Expected: PASS

### Task 3: Wire the repeat helper into playlist input handling

**Files:**
- Modify: `spotify-ui-rs/src/input.rs`

**Step 1: Keep the helper tests red-green locked**

Use the repeat helper tests from Task 2 as the behavior contract.

**Step 2: Write minimal implementation**

Update playlist input handling so:
- `ABS_HAT0Y` immediate press sends one move and arms repeat
- release cancels repeat
- overlay dismissal stops repeat
- normal mode input remains unchanged

**Step 3: Run targeted verification**

Run: `cargo test playlist_selection playlist_repeat`
Expected: PASS

### Task 4: Run full verification

**Files:**
- Modify: `spotify-ui-rs/src/main.rs`
- Modify: `spotify-ui-rs/src/input.rs`

**Step 1: Run the full Rust test suite**

Run: `cargo test`
Expected: all tests pass

**Step 2: Rebuild package if device verification is needed**

Run: `bash scripts/package.sh`
Expected: exit 0 and fresh package artifacts
