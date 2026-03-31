# FFmpeg Lite Rename Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rename SideB's bundled FFmpeg-compatible transcoder from `ffmpeg-full` to `ffmpeg-lite` across packaging, runtime, code, and docs without changing system ffmpeg behavior.

**Architecture:** The bundled transcoder is staged from `package/SideB.pak`, copied into each release package, then copied again to `/tmp` at launch where Rust and `yt-dlp` use it. The implementation keeps that flow unchanged and only renames the bundled artifact and its references. System `/usr/bin/ffmpeg` remains a separate path used for embedded cover extraction.

**Tech Stack:** Bash packaging scripts, POSIX shell launchers, Rust unit tests, ripgrep verification.

---

### Task 1: Lock the bundled/runtime naming behavior with tests

**Files:**
- Modify: `spotify-ui-rs/src/constants.rs`
- Test: `spotify-ui-rs/src/constants.rs`

**Step 1: Write the failing test**

Add a unit test that asserts `FFMPEG_TRANSCODER_BIN == "/tmp/ffmpeg-lite"` and `SYSTEM_FFMPEG_BIN == "/usr/bin/ffmpeg"`.

**Step 2: Run test to verify it fails**

Run: `cargo test constants::tests::bundled_ffmpeg_uses_lite_name`
Expected: FAIL because the bundled constant still points to `/tmp/ffmpeg-full`

**Step 3: Write minimal implementation**

Rename the bundled constant value to `/tmp/ffmpeg-lite` and add the smallest matching test module.

**Step 4: Run test to verify it passes**

Run: `cargo test constants::tests::bundled_ffmpeg_uses_lite_name`
Expected: PASS

### Task 2: Rename bundled packaging and runtime references

**Files:**
- Modify: `scripts/package.sh`
- Modify: `package/SideB.pak/launch.sh`
- Modify: `packaging/nextui/launch.sh`
- Modify: `packaging/stock/launch.sh`
- Modify: `packaging/crossmix/launch.sh`
- Modify: `.gitignore`

**Step 1: Write the failing test**

Use the Task 1 failing test as the behavior lock for the runtime name before editing packaging references.

**Step 2: Run test to verify it fails**

Run: `cargo test constants::tests::bundled_ffmpeg_uses_lite_name`
Expected: FAIL before code changes

**Step 3: Write minimal implementation**

Rename packaged file references from `ffmpeg-full` to `ffmpeg-lite`, including copy, chmod, existence checks, and `/tmp` staging logic.

**Step 4: Run verification**

Run: `rg -n --hidden --glob '!.git' 'ffmpeg-full|/tmp/ffmpeg-full' .`
Expected: no bundled/runtime references remain

### Task 3: Rename bundled docs and metadata references

**Files:**
- Modify: `README.md`
- Modify: `packaging/shared/LICENSES/THIRD_PARTY_SOURCES.md`

**Step 1: Write the failing test**

Use a text verification step to define success: no bundled `ffmpeg-full` references remain in docs.

**Step 2: Run text verification to observe current failures**

Run: `rg -n 'ffmpeg-full' README.md packaging/shared/LICENSES/THIRD_PARTY_SOURCES.md`
Expected: matches are returned

**Step 3: Write minimal implementation**

Update docs so the bundled artifact is consistently called `ffmpeg-lite` while system ffmpeg references remain descriptive and unchanged.

**Step 4: Run verification**

Run: `rg -n 'ffmpeg-full' README.md packaging/shared/LICENSES/THIRD_PARTY_SOURCES.md`
Expected: no matches

### Task 4: Verify the full rename end-to-end

**Files:**
- Modify: `package/SideB.pak/ffmpeg-lite`

**Step 1: Ensure the packaged binary name matches the code**

Rename the local bundled binary file from `package/SideB.pak/ffmpeg-full` to `package/SideB.pak/ffmpeg-lite`.

**Step 2: Run focused tests**

Run: `cargo test bundled_ffmpeg_uses_lite_name embedded_cover_extraction_uses_system_ffmpeg`
Expected: PASS

**Step 3: Run project-wide verification**

Run: `rg -n --hidden --glob '!.git' 'ffmpeg-full|/tmp/ffmpeg-full' .`
Expected: no matches

Run: `ls -lh package/SideB.pak/ffmpeg-lite`
Expected: bundled binary exists at the lite name and remains around 3 MB
