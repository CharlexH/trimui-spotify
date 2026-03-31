# SideB Bundled FFmpeg Lite Design

**Date:** 2026-03-31

**Goal:** Rename the bundled FFmpeg-compatible transcoder from `ffmpeg-full` to `ffmpeg-lite` everywhere in the project while keeping the device system ffmpeg behavior unchanged.

## Scope

- Rename the packaged binary at `package/SideB.pak/` to `ffmpeg-lite`.
- Update packaging scripts and runtime launchers to stage `/tmp/ffmpeg-lite`.
- Update Rust constants and code paths that point to the bundled transcoder.
- Update docs and packaging metadata so the bundled binary is consistently described as `ffmpeg-lite`.
- Leave all system ffmpeg behavior unchanged, including `/usr/bin/ffmpeg` usage for local cover extraction.

## Constraints

- The bundled binary is already a trimmed build around 3.2 MB and should remain the bundled artifact.
- The system ffmpeg path must continue to be treated as a separate dependency and naming concept.
- Rename consistency matters more than backward compatibility with `ffmpeg-full`.

## Chosen Approach

Use a full project-wide rename for the bundled transcoder references and keep system ffmpeg references intact.

## Affected Areas

- `package/SideB.pak/`
- `scripts/package.sh`
- `packaging/*/launch.sh`
- `spotify-ui-rs/src/constants.rs`
- project documentation and packaging notices
- ignore rules for packaged binaries

## Testing

- Add a small unit test that locks the bundled transcoder path to `/tmp/ffmpeg-lite`.
- Keep the existing unit test that verifies embedded cover extraction uses `/usr/bin/ffmpeg`.
- Run focused Rust tests plus a project-wide search to confirm no `ffmpeg-full` bundled references remain.
