# Runtime Logging Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add targeted, low-noise runtime logs for SideB download, local playback, and local import flows so real-device troubleshooting is practical from `/tmp/sideb.log`.

**Architecture:** Keep the existing direct `eprintln!` approach and add only milestone logs around state transitions and subprocess boundaries. Shared formatting concerns such as stderr summarization and byte-size formatting live in a small utility module so all runtime paths produce consistent log output without introducing a full logging framework.

**Tech Stack:** Rust binary crate, stdlib subprocess APIs, focused unit tests, existing device packaging scripts.

---

### Task 1: Add shared log helper coverage first

**Files:**
- Create: `spotify-ui-rs/src/log_utils.rs`
- Modify: `spotify-ui-rs/src/main.rs`
- Test: `spotify-ui-rs/src/log_utils.rs`

**Step 1: Write the failing test**

Add unit tests that define the expected logging helper behavior:
- stderr summaries should use the last non-empty line
- long lines should be truncated for logs
- byte counts should format into human-readable binary units

**Step 2: Run test to verify it fails**

Run: `cargo test log_utils`
Expected: FAIL because the initial helper behavior is incomplete or incorrect

**Step 3: Write minimal implementation**

Implement the helper functions and wire the module into the crate root.

**Step 4: Run test to verify it passes**

Run: `cargo test log_utils`
Expected: PASS

### Task 2: Enhance download lifecycle logs

**Files:**
- Modify: `spotify-ui-rs/src/download.rs`
- Test: `spotify-ui-rs/src/log_utils.rs`

**Step 1: Use the Task 1 helper tests as the red phase**

The shared helper tests lock the formatting behavior used in download logs.

**Step 2: Write minimal implementation**

Add milestone logs for queueing, command launch, retries, output path selection, yt-dlp failures with exit status and stderr summary, ffprobe outcomes, success details, and cover-art handling.

**Step 3: Run targeted verification**

Run: `cargo test log_utils ffmpeg`
Expected: PASS

### Task 3: Enhance local playback and import logs

**Files:**
- Modify: `spotify-ui-rs/src/local_player.rs`
- Modify: `spotify-ui-rs/src/local_import.rs`
- Test: `spotify-ui-rs/src/log_utils.rs`

**Step 1: Use the existing tests plus Task 1 helper tests as the safety net**

Keep `embedded_cover_extraction_uses_system_ffmpeg` and the logging helper tests green while enhancing logs.

**Step 2: Write minimal implementation**

Add logs for import scan discovery, metadata source, move target, cover source, playback target selection, pipeline launch, auto-advance, and playback state transitions.

**Step 3: Run targeted verification**

Run: `cargo test log_utils embedded_cover_extraction_uses_system_ffmpeg`
Expected: PASS

### Task 4: Verify on build and device

**Files:**
- Modify: `scripts/package.sh`

**Step 1: Build the package**

Run: `bash scripts/package.sh`
Expected: exit 0 and fresh `dist/SideB-1.0.3-nextui.zip`

**Step 2: Deploy and inspect logs on device**

Push the NextUI package to `/mnt/SDCARD/Tools/tg5040/SideB.pak`, launch it, trigger a favorite download, and inspect `/tmp/sideb.log`.

**Step 3: Confirm the new log coverage**

Verify the device log now shows each major stage of the download lifecycle, plus enough context for playback and import troubleshooting.
