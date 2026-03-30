# Favorites Overlay Copy Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Unify the Favorites overlay title and footer copy/style with the main playback UI while preserving the existing divider line and list layout.

**Architecture:** Keep the Favorites overlay rendering structure unchanged and isolate the copy/styling changes behind small helper functions in `playlist_view.rs`. This keeps the visual tweak easy to test and easy to retune without disturbing playlist row geometry.

**Tech Stack:** Rust, software framebuffer UI, existing `FontSet` text rendering helpers.

---

### Task 1: Lock the new copy and footer alignment in tests

**Files:**
- Modify: `spotify-ui-rs/src/playlist_view.rs`
- Test: `spotify-ui-rs/src/playlist_view.rs`

**Step 1: Write the failing test**

Add tests for:
- header title formatting: `FAV LIST (n)`
- footer copy: `NAVIGATE (↑/↓)   PLAY (A)   DELETE (X)   BACK (B)`
- footer baseline/color values matching the playback-page helper style

**Step 2: Run test to verify it fails**

Run: `cargo test playlist_view --manifest-path spotify-ui-rs/Cargo.toml`
Expected: FAIL because the helpers or expected values do not exist yet.

**Step 3: Write minimal implementation**

Add helper functions/constants for:
- playlist header title
- playlist footer copy
- playlist footer baseline and gray color

Update `render_playlist_overlay()` to use those helpers.

**Step 4: Run test to verify it passes**

Run: `cargo test playlist_view --manifest-path spotify-ui-rs/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add docs/plans/2026-03-30-favorites-overlay-copy-design.md spotify-ui-rs/src/playlist_view.rs
git commit -m "fix: align favorites overlay copy with playback ui"
```
