//! Local integration test for the multi-candidate search + scoring pipeline.
//!
//! Usage:
//!   cargo run --example test_download
//!
//! Requires: yt-dlp, ffmpeg, ffprobe in PATH.
//! Does NOT require the device or go-librespot.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

const CANDIDATE_COUNT: usize = 5;
const DURATION_REJECT_THRESHOLD_MS: i64 = 15_000;

// ---------------------------------------------------------------------------
// Types (mirrored from src/download.rs for standalone use)
// ---------------------------------------------------------------------------

struct DownloadRequest {
    track_name: String,
    artist_name: String,
    spotify_duration_ms: Option<i64>,
}

struct SearchCandidate {
    id: String,
    title: String,
    duration_secs: Option<f64>,
    channel: Option<String>,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

fn search_candidates(query: &str, count: usize) -> Vec<SearchCandidate> {
    let search_term = format!("ytsearch{}:{}", count, query);
    let output = Command::new("yt-dlp")
        .args(["--dump-single-json", "--flat-playlist", "--no-warnings", &search_term])
        .output()
        .expect("yt-dlp not found in PATH");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("  [ERROR] yt-dlp search failed: {}", stderr.trim());
        return Vec::new();
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();

    let entries = match val.get("entries").and_then(|e| e.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    entries
        .iter()
        .filter_map(|entry| {
            let id = entry.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let title = entry.get("title").and_then(|v| v.as_str())?.to_string();
            if title.trim().is_empty() { return None; }
            let duration_secs = entry.get("duration").and_then(|v| v.as_f64());
            let channel = entry
                .get("channel")
                .or_else(|| entry.get("uploader"))
                .and_then(|v| v.as_str())
                .map(String::from);
            Some(SearchCandidate { id, title, duration_secs, channel })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Scoring (exact copy from src/download.rs)
// ---------------------------------------------------------------------------

fn title_similarity(candidate_title: &str, reference_title: &str) -> f64 {
    let normalize = |s: &str| -> HashSet<String> {
        s.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(String::from)
            .collect()
    };
    let candidate_words = normalize(candidate_title);
    let reference_words = normalize(reference_title);
    if reference_words.is_empty() { return 0.0; }
    let intersection = candidate_words.intersection(&reference_words).count();
    intersection as f64 / reference_words.len() as f64
}

fn score_candidate(candidate: &SearchCandidate, request: &DownloadRequest) -> f64 {
    let mut score = 0.0;

    if let (Some(cand_secs), Some(spotify_ms)) = (candidate.duration_secs, request.spotify_duration_ms) {
        let cand_ms = (cand_secs * 1000.0) as i64;
        let diff_ms = (spotify_ms - cand_ms).abs();
        score += if diff_ms <= 2_000 { 40.0 }
            else if diff_ms <= 5_000 { 30.0 }
            else if diff_ms <= 10_000 { 15.0 }
            else if diff_ms <= 30_000 { 5.0 }
            else { 0.0 };
    }

    score += title_similarity(&candidate.title, &request.track_name) * 25.0;

    if let Some(ref ch) = candidate.channel {
        if ch.ends_with(" - Topic") { score += 15.0; }
    }

    let cand_lower = candidate.title.to_lowercase();
    let req_lower = request.track_name.to_lowercase();
    for keyword in &["cover", "remix", "live", "karaoke", "instrumental", "acoustic"] {
        if cand_lower.contains(keyword) && !req_lower.contains(keyword) {
            score -= 15.0;
        }
    }

    score
}

// ---------------------------------------------------------------------------
// Download + Validation
// ---------------------------------------------------------------------------

fn download_candidate(id: &str, output_path: &Path) -> bool {
    let url = format!("https://www.youtube.com/watch?v={}", id);
    let output = Command::new("yt-dlp")
        .args([
            "-x", "--audio-format", "mp3", "--audio-quality", "5",
            "--no-playlist",
            "-o", &output_path.to_string_lossy(),
            &url,
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => true,
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            eprintln!("  [FAIL] download failed: {}", stderr.chars().take(200).collect::<String>());
            false
        }
        Err(e) => { eprintln!("  [FAIL] yt-dlp error: {e}"); false }
    }
}

fn probe_duration(path: &Path) -> Option<i64> {
    let output = Command::new("ffprobe")
        .args(["-v", "quiet", "-show_entries", "format=duration", "-of", "csv=p=0"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() { return None; }
    let s = String::from_utf8_lossy(&output.stdout);
    let secs: f64 = s.trim().parse().ok()?;
    Some((secs * 1000.0) as i64)
}

fn validate_duration(file: &Path, spotify_ms: Option<i64>) -> Result<i64, String> {
    let file_ms = probe_duration(file).ok_or("ffprobe failed")?;
    if let Some(expected) = spotify_ms {
        let diff = (expected - file_ms).abs();
        if diff > DURATION_REJECT_THRESHOLD_MS {
            return Err(format!("duration mismatch: spotify={}ms file={}ms diff={}ms", expected, file_ms, diff));
        }
    }
    Ok(file_ms)
}

// ---------------------------------------------------------------------------
// Test cases
// ---------------------------------------------------------------------------

struct TestCase {
    artist: &'static str,
    track: &'static str,
    spotify_duration_ms: i64,
}

fn main() {
    let test_cases = vec![
        TestCase { artist: "Arianne", track: "Komm, Susser Tod", spotify_duration_ms: 467_000 },
        TestCase { artist: "Radiohead", track: "Creep", spotify_duration_ms: 236_000 },
        TestCase { artist: "久石譲", track: "Summer", spotify_duration_ms: 245_000 },
        TestCase { artist: "Adele", track: "Hello", spotify_duration_ms: 295_000 },
        TestCase { artist: "Jay Chou", track: "Mojito", spotify_duration_ms: 194_000 },
    ];

    let tmp_dir = std::env::temp_dir().join("sideb-test-download");
    let _ = std::fs::create_dir_all(&tmp_dir);

    let download_mode = std::env::args().any(|a| a == "--download");

    println!("=== SideB Download Pipeline Test ===");
    println!("Mode: {}", if download_mode { "search + download + validate" } else { "search + score only (use --download to also download)" });
    println!();

    for tc in &test_cases {
        let query = format!("{} - {}", tc.artist, tc.track);
        println!("--- {} ---", query);
        println!("  Spotify duration: {}ms ({:.1}s)", tc.spotify_duration_ms, tc.spotify_duration_ms as f64 / 1000.0);

        let req = DownloadRequest {
            track_name: tc.track.to_string(),
            artist_name: tc.artist.to_string(),
            spotify_duration_ms: Some(tc.spotify_duration_ms),
        };

        let candidates = search_candidates(&query, CANDIDATE_COUNT);
        if candidates.is_empty() {
            println!("  [WARN] No candidates found!");
            println!();
            continue;
        }

        let mut scored: Vec<(f64, &SearchCandidate)> = candidates.iter().map(|c| (score_candidate(c, &req), c)).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        for (i, (sc, cand)) in scored.iter().enumerate() {
            let dur_str = cand.duration_secs
                .map(|d| {
                    let diff = tc.spotify_duration_ms - (d * 1000.0) as i64;
                    format!("{:.1}s (diff={:+.1}s)", d, diff as f64 / -1000.0)
                })
                .unwrap_or_else(|| "unknown".into());
            let marker = if i == 0 { " <-- BEST" } else { "" };
            println!(
                "  #{} score={:5.1} | dur={} | ch={} | \"{}\"{}",
                i + 1,
                sc,
                dur_str,
                cand.channel.as_deref().unwrap_or("?"),
                cand.title,
                marker
            );
        }

        if download_mode {
            let (best_score, best) = &scored[0];
            let safe_name = query.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            let output_path = tmp_dir.join(format!("{}.mp3", safe_name));
            let _ = std::fs::remove_file(&output_path);

            println!("  Downloading best candidate (score={:.1}, id={})...", best_score, best.id);
            if download_candidate(&best.id, &output_path) {
                match validate_duration(&output_path, Some(tc.spotify_duration_ms)) {
                    Ok(file_ms) => {
                        let diff = (tc.spotify_duration_ms - file_ms).abs();
                        println!("  [OK] Downloaded & validated: file={}ms diff={}ms", file_ms, diff);
                    }
                    Err(reason) => {
                        println!("  [REJECT] {}", reason);
                        let _ = std::fs::remove_file(&output_path);
                    }
                }
            }
        }

        println!();
    }

    if download_mode {
        println!("Downloaded files in: {}", tmp_dir.display());
    }
}
