# Playback Footer Copy Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Update the playback page footer hints to match the revised six-label copy from the approved reference image.

**Architecture:** Keep the existing footer rendering pipeline and spacing logic in `render.rs`, but replace the hardcoded label array with a small helper that returns the new six labels. This isolates the copy change from layout behavior.

**Tech Stack:** Rust, framebuffer text rendering via `FontSet`.

---

### Task 1: Lock the approved footer copy in tests

**Files:**
- Modify: `spotify-ui-rs/src/render.rs`
- Create: `docs/plans/2026-03-30-playback-footer-copy-design.md`

**Step 1: Write the failing test**

Add a unit test that asserts the playback footer label list is exactly:
- `PREV/NEXT (←/→)`
- `VOL+/- (↑/↓)`
- `PLAY/PAUSE (A)`
- `FAV (X)`
- `LIST (Y)`
- `EXIT (B)`

**Step 2: Run test to verify it fails**

Run: `cargo test playback_footer_labels_match_reference_copy --manifest-path spotify-ui-rs/Cargo.toml`
Expected: FAIL because the helper or updated labels do not exist yet.

**Step 3: Write minimal implementation**

Extract the footer label list into a helper and update `rebuild_base_scene()` to use it.

**Step 4: Run test to verify it passes**

Run: `cargo test playback_footer_labels_match_reference_copy --manifest-path spotify-ui-rs/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add docs/plans/2026-03-30-playback-footer-copy-design.md spotify-ui-rs/src/render.rs
git commit -m "fix: update playback footer copy"
```
