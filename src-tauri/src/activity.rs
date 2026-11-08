//! Opt-in, local-only Roblox Player activity detection.
//!
//! Reads only a bounded tail of recent Player logs from Roblox's documented
//! macOS/Windows log directories. Never transfers or returns raw log lines,
//! usernames, network addresses, cookies, or machine paths to the webview.
//! Roblox's log messages can change without notice; join lines are attempts,
//! not authoritative proof that a game fully loaded.

use chrono::{DateTime, NaiveDateTime};
use serde::Serialize;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOG_FILES: usize = 12;
const MAX_TAIL_BYTES: u64 = 512 * 1024;
const MAX_SESSIONS: usize = 80;
const MAX_LOOKBACK_MS: i64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSession {
    place_id: u64,
    server_id: String,
    joined_at: i64,
    ended_at: Option<i64>,
}

fn timestamp_ms(value: SystemTime) -> Option<i64> {
    let ms = value.duration_since(UNIX_EPOCH).ok()?.as_millis();
    i64::try_from(ms).ok()
}

fn line_timestamp_ms(line: &str) -> Option<i64> {
    let token = line.split_whitespace().next()?
        .trim_matches(|ch: char| matches!(ch, '[' | ']' | ',' | '(' | ')'));
    if let Ok(parsed) = DateTime::parse_from_rfc3339(token) {
        return Some(parsed.timestamp_millis());
    }
    // Some Player builds use a timezone-less ISO timestamp; keep it UTC
    // instead of interpreting it with the user's local timezone.
    let naive = NaiveDateTime::parse_from_str(token, "%Y-%m-%dT%H:%M:%S%.f").ok()?;
    Some(naive.and_utc().timestamp_millis())
}

fn joined_place(line: &str) -> Option<(u64, String)> {
    let tail = line.split_once("! Joining game '")?.1;
    let (server, after) = tail.split_once("' place ")?;
    let uuid = uuid::Uuid::parse_str(server).ok()?.to_string();
    let raw_place = after.split_whitespace().next()?;
    if raw_place.is_empty() || !raw_place.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let place_id: u64 = raw_place.parse().ok()?;
    if place_id == 0 || place_id > 9_007_199_254_740_991 {
        return None;
    }
    Some((place_id, uuid))
}

fn is_leave_line(line: &str) -> bool {
    line.contains("Time to disconnect replication data:")
        || line.contains("leaveUGCGameInternal")
}

fn parse_sessions(content: &str, since_ms: i64, now_ms: i64) -> Vec<PlayerSession> {
    let mut sessions: Vec<PlayerSession> = Vec::new();
    let mut current: Option<usize> = None;
    for line in content.lines() {
        let Some(at) = line_timestamp_ms(line) else { continue; };
        if at < since_ms || at > now_ms + 60_000 { continue; }

        if let Some((place_id, server_id)) = joined_place(line) {
            if let Some(idx) = current {
                if sessions[idx].ended_at.is_none() && at >= sessions[idx].joined_at {
                    sessions[idx].ended_at = Some(at);
                }
            }
            sessions.push(PlayerSession { place_id, server_id, joined_at: at, ended_at: None });
            current = Some(sessions.len() - 1);
        } else if is_leave_line(line) {
            if let Some(idx) = current.take() {
                if at >= sessions[idx].joined_at && sessions[idx].ended_at.is_none() {
                    sessions[idx].ended_at = Some(at);
                }
            }
        }
    }
    sessions
}

fn player_log_directory() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from(std::env::var_os("HOME")?)
            .join("Library").join("Logs").join("Roblox"))
    }
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?)
            .join("Roblox").join("logs"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn looks_like_player_log(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else { return false; };
    let name = name.to_ascii_lowercase();
    // Never parse Studio-only or crash/dump files.
    if name.contains("studio") || name.contains("crash") || name.contains("dump") {
        return false;
    }
    (name.contains("player") || name.starts_with("log_") || name.starts_with("rbx-"))
        && (name.ends_with(".log") || name.ends_with(".txt"))
}

fn read_log_tail(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let offset = len.saturating_sub(MAX_TAIL_BYTES);
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::with_capacity((len - offset) as usize);
    file.take(MAX_TAIL_BYTES).read_to_end(&mut bytes)?;
    let contents = String::from_utf8_lossy(&bytes);
    // Skip the first partial line when starting in the middle of a file.
    if offset > 0 {
        Ok(contents.split_once('\n').map(|(_, tail)| tail).unwrap_or("").to_owned())
    } else {
        Ok(contents.into_owned())
    }
}

fn scan_log_directory(directory: &Path, since_ms: i64, now_ms: i64) -> Vec<PlayerSession> {
    let Ok(entries) = fs::read_dir(directory) else { return vec![]; };
    let mut candidates: Vec<_> = entries.flatten()
        .filter_map(|entry| {
            // Do not follow symlinks to files outside Roblox's log folder.
            if !entry.file_type().ok()?.is_file() { return None; }
            let path = entry.path();
            if !looks_like_player_log(&path) { return None; }
            let modified = timestamp_ms(entry.metadata().ok()?.modified().ok()?)?;
            if modified < since_ms || modified > now_ms + 60_000 { return None; }
            Some((modified, path))
        }).collect();
    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    candidates.truncate(MAX_LOG_FILES);

    let mut output = Vec::new();
    for (_, path) in candidates {
        if let Ok(text) = read_log_tail(&path) {
            output.extend(parse_sessions(&text, since_ms, now_ms));
        }
    }
    output.sort_by(|a, b| b.joined_at.cmp(&a.joined_at));
    output.dedup_by(|a, b| a.place_id == b.place_id
        && a.server_id == b.server_id && a.joined_at == b.joined_at);
    output.truncate(MAX_SESSIONS);
    output
}

#[tauri::command]
pub fn recent_player_sessions(since_ms: i64) -> Result<Vec<PlayerSession>, String> {
    let now = timestamp_ms(SystemTime::now()).ok_or("Clock is unavailable.")?;
    // The webview cannot turn this into an arbitrary-path log reader.
    if since_ms < now - MAX_LOOKBACK_MS || since_ms > now + 60_000 {
        return Err("Tracking start time is outside the supported seven-day window.".into());
    }
    let Some(dir) = player_log_directory() else { return Ok(vec![]); };
    Ok(scan_log_directory(&dir, since_ms.min(now), now))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_join_and_leave_without_serializing_raw_log_data() {
        let input = concat!(
            "2026-11-08T18:00:00.200Z [FLog::Output] ! Joining game '550e8400-e29b-41d4-a716-446655440000' place 1818 at 10.2.3.4\n",
            "2026-11-08T18:03:00.000Z [FLog::Network] Time to disconnect replication data: 125\n",
        );
        let start = line_timestamp_ms(input.lines().next().unwrap()).unwrap();
        let result = parse_sessions(input, start - 500, start + 500_000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].place_id, 1818);
        assert_eq!(result[0].server_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(result[0].ended_at, Some(start + 179_800));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("10.2.3.4"));
        assert!(!json.contains("FLog"));
    }

    #[test]
    fn rejects_invalid_or_old_joins() {
        let input = concat!(
            "2026-11-08T17:30:00Z [FLog::Output] ! Joining game 'invalid' place 1 at 1.1.1.1\n",
            "2026-11-08T17:30:01Z [FLog::Output] ! Joining game '550e8400-e29b-41d4-a716-446655440000' place 0 at 1.1.1.1\n",
            "2026-11-08T17:30:02Z [FLog::Output] ! Joining game '550e8400-e29b-41d4-a716-446655440000' place 5 at 1.1.1.1\n"
        );
        let after = line_timestamp_ms("2026-11-08T17:31:00Z [FLog::Output]").unwrap();
        assert!(parse_sessions(input, after, after + 60_000).is_empty());
        assert!(!looks_like_player_log(Path::new("RobloxStudio_2026.log")));
        assert!(looks_like_player_log(Path::new("0.500_Player_123.log")));
    }

    #[test]
    fn implicit_close_when_next_join_begins() {
        let input = concat!(
            "2026-11-08T18:00:00Z [FLog::Output] ! Joining game '550e8400-e29b-41d4-a716-446655440000' place 10 at 0.0.0.0\n",
            "2026-11-08T18:04:00Z [FLog::Output] ! Joining game '123e4567-e89b-12d3-a456-426614174000' place 20 at 0.0.0.0\n",
        );
        let start = line_timestamp_ms(input.lines().next().unwrap()).unwrap();
        let result = parse_sessions(input, start, start + 500_000);
        assert_eq!(result[0].ended_at, Some(start + 240_000));
        assert_eq!(result[1].ended_at, None);
    }
}
