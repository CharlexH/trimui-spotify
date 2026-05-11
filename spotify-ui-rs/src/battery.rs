use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app::AppState;

pub const POLL_INTERVAL: Duration = Duration::from_secs(5);
const DEFAULT_PERC_BAT_PATH: &str = "/tmp/percBat";
const DEFAULT_POWER_SUPPLY_ROOT: &str = "/sys/class/power_supply";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatterySnapshot {
    pub percent: Option<u8>,
    pub charging: bool,
}

pub fn refresh_app_state(app_state: &Arc<Mutex<AppState>>) {
    let snapshot = read_battery_snapshot();
    app_state
        .lock()
        .unwrap()
        .set_battery_snapshot(snapshot.percent, snapshot.charging);
}

pub fn run(app_state: Arc<Mutex<AppState>>, quit: Arc<AtomicBool>) {
    while !quit.load(Ordering::Relaxed) {
        refresh_app_state(&app_state);
        sleep_until_next_poll(&quit);
    }
}

fn sleep_until_next_poll(quit: &AtomicBool) {
    let step = Duration::from_millis(250);
    let mut slept = Duration::ZERO;
    while slept < POLL_INTERVAL {
        if quit.load(Ordering::Relaxed) {
            return;
        }
        let remaining = POLL_INTERVAL.saturating_sub(slept);
        let sleep_for = remaining.min(step);
        std::thread::sleep(sleep_for);
        slept += sleep_for;
    }
}

pub fn read_battery_percent() -> Option<u8> {
    read_battery_snapshot().percent
}

pub fn read_battery_snapshot() -> BatterySnapshot {
    read_battery_snapshot_from_paths(
        Path::new(DEFAULT_PERC_BAT_PATH),
        Path::new(DEFAULT_POWER_SUPPLY_ROOT),
    )
}

fn parse_capacity(raw: &str) -> Option<u8> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let value = raw.parse::<i16>().ok()?;
    if (0..=100).contains(&value) {
        Some(value as u8)
    } else {
        None
    }
}

fn read_battery_percent_from_paths(perc_bat: &Path, power_supply_root: &Path) -> Option<u8> {
    read_battery_snapshot_from_paths(perc_bat, power_supply_root).percent
}

fn read_battery_snapshot_from_paths(perc_bat: &Path, power_supply_root: &Path) -> BatterySnapshot {
    let sysfs = read_power_supply_root(power_supply_root);
    BatterySnapshot {
        percent: read_capacity_file(perc_bat).or(sysfs.percent),
        charging: sysfs.charging,
    }
}

fn read_capacity_file(path: &Path) -> Option<u8> {
    let raw = fs::read_to_string(path).ok()?;
    parse_capacity(&raw)
}

fn read_charging_status(path: &Path) -> Option<bool> {
    let raw = fs::read_to_string(path).ok()?;
    Some(raw.trim().eq_ignore_ascii_case("Charging"))
}

fn read_power_supply_root(root: &Path) -> BatterySnapshot {
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();

    let mut battery = BatterySnapshot {
        percent: None,
        charging: false,
    };
    let mut fallback = BatterySnapshot {
        percent: None,
        charging: false,
    };
    let mut saw_battery = false;
    for dir in dirs {
        let capacity = read_capacity_file(&dir.join("capacity"));
        let charging = read_charging_status(&dir.join("status")).unwrap_or(false);

        let supply_type = fs::read_to_string(dir.join("type")).unwrap_or_default();
        if supply_type.trim().eq_ignore_ascii_case("Battery") {
            saw_battery = true;
            if battery.percent.is_none() {
                battery.percent = capacity;
            }
            battery.charging |= charging;
            continue;
        }

        if fallback.percent.is_none() {
            fallback.percent = capacity;
        }
        fallback.charging |= charging;
    }

    if saw_battery {
        battery
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("sideb-battery-{name}-{suffix}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_capacity_accepts_trimmed_percent_values() {
        assert_eq!(parse_capacity("87\n"), Some(87));
        assert_eq!(parse_capacity("0"), Some(0));
        assert_eq!(parse_capacity("100"), Some(100));
    }

    #[test]
    fn poll_interval_keeps_charger_state_visibly_fresh() {
        assert!(POLL_INTERVAL <= Duration::from_secs(5));
    }

    #[test]
    fn parse_capacity_rejects_invalid_values() {
        assert_eq!(parse_capacity("101"), None);
        assert_eq!(parse_capacity("-1"), None);
        assert_eq!(parse_capacity("charging"), None);
        assert_eq!(parse_capacity(""), None);
    }

    #[test]
    fn perc_bat_file_wins_over_sysfs_capacity() {
        let dir = temp_dir("priority");
        let perc_bat = dir.join("percBat");
        let root = dir.join("power_supply");
        let bat0 = root.join("BAT0");
        fs::create_dir_all(&bat0).unwrap();
        fs::write(&perc_bat, "88\n").unwrap();
        fs::write(bat0.join("type"), "Battery\n").unwrap();
        fs::write(bat0.join("capacity"), "42\n").unwrap();

        assert_eq!(read_battery_percent_from_paths(&perc_bat, &root), Some(88));
    }

    #[test]
    fn sysfs_reader_prefers_power_supply_entries_typed_as_battery() {
        let dir = temp_dir("sysfs");
        let missing_perc_bat = dir.join("missing-percBat");
        let root = dir.join("power_supply");
        let ac = root.join("AC");
        let bat0 = root.join("BAT0");
        fs::create_dir_all(&ac).unwrap();
        fs::create_dir_all(&bat0).unwrap();
        fs::write(ac.join("type"), "Mains\n").unwrap();
        fs::write(ac.join("capacity"), "99\n").unwrap();
        fs::write(bat0.join("type"), "Battery\n").unwrap();
        fs::write(bat0.join("capacity"), "55\n").unwrap();

        assert_eq!(
            read_battery_percent_from_paths(&missing_perc_bat, &root),
            Some(55)
        );
    }

    #[test]
    fn snapshot_uses_perc_bat_for_percent_and_sysfs_for_charging_state() {
        let dir = temp_dir("snapshot");
        let perc_bat = dir.join("percBat");
        let root = dir.join("power_supply");
        let bat0 = root.join("BAT0");
        fs::create_dir_all(&bat0).unwrap();
        fs::write(&perc_bat, "88\n").unwrap();
        fs::write(bat0.join("type"), "Battery\n").unwrap();
        fs::write(bat0.join("capacity"), "42\n").unwrap();
        fs::write(bat0.join("status"), "Charging\n").unwrap();

        assert_eq!(
            read_battery_snapshot_from_paths(&perc_bat, &root),
            BatterySnapshot {
                percent: Some(88),
                charging: true
            }
        );
    }

    #[test]
    fn snapshot_ignores_stale_external_power_online_when_battery_is_full() {
        let dir = temp_dir("stale-external-online");
        let perc_bat = dir.join("percBat");
        let root = dir.join("power_supply");
        let bat0 = root.join("BAT0");
        let usb = root.join("USB");
        fs::create_dir_all(&bat0).unwrap();
        fs::create_dir_all(&usb).unwrap();
        fs::write(&perc_bat, "100\n").unwrap();
        fs::write(bat0.join("type"), "Battery\n").unwrap();
        fs::write(bat0.join("capacity"), "100\n").unwrap();
        fs::write(bat0.join("status"), "Full\n").unwrap();
        fs::write(usb.join("type"), "USB\n").unwrap();
        fs::write(usb.join("online"), "1\n").unwrap();

        assert_eq!(
            read_battery_snapshot_from_paths(&perc_bat, &root),
            BatterySnapshot {
                percent: Some(100),
                charging: false
            }
        );
    }
}
