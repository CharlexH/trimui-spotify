# SideB Runtime Logging Design

**Date:** 2026-03-31

**Goal:** Improve runtime observability for downloads, local playback, and local imports without turning SideB logs into a noisy debug stream.

## Scope

- Add required lifecycle logs for the download queue and yt-dlp/ffmpeg-lite processing path.
- Add required lifecycle logs for local playback startup, pipeline failures, and track transitions.
- Add required lifecycle logs for local import scanning, metadata resolution, file moves, and cover handling.
- Keep the current `eprintln!` logging style and avoid adding a separate logging framework.

## Constraints

- Logs should help diagnose real-device failures from `/tmp/sideb.log`.
- Success logs should be short and concrete.
- Failure logs should include the minimum useful context: track or file identity, path, exit status, and a compact stderr summary.
- High-frequency render or polling code must not gain extra logs.

## Chosen Approach

Use a small shared log helper module for byte-size formatting and stderr summarization, then add milestone logs in each runtime pipeline:

- `download`: queue, start, command launch, retry, success, failure, cover outcome, metadata probe outcome
- `local_player`: playback target selection, pipeline launch, pause/resume/stop with track context, auto-advance
- `local_import`: scan discovery, metadata source, move target, cover source, success or failure

## Testing

- Add focused unit tests for the shared log helper behavior.
- Run targeted Rust tests for logging helper coverage plus the existing ffmpeg path tests.
- Rebuild and redeploy to the NextUI device package, then verify that runtime logs now expose the full download chain.
