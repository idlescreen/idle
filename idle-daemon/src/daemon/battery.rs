// SPDX-License-Identifier: MIT

//! Battery / AC heuristics that force inhibit when on battery.

/// True when the host looks like it is running on battery (AC offline or
/// discharging). Used to treat power-as-inhibit so idle savers stay off.
pub fn is_on_battery() -> bool {
    let path = std::path::Path::new("/sys/class/power_supply");
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };

    let mut has_ac = false;
    let mut ac_online = true;
    let mut battery_discharging = false;

    for entry in entries.flatten() {
        let p = entry.path();
        let Ok(t) = std::fs::read_to_string(p.join("type")) else {
            continue;
        };
        let type_str = t.trim();
        if type_str == "Mains" {
            has_ac = true;
            if let Ok(o) = std::fs::read_to_string(p.join("online")) {
                ac_online = o.trim() != "0";
            }
        } else if type_str == "Battery"
            && let Ok(s) = std::fs::read_to_string(p.join("status"))
            && s.trim() == "Discharging"
        {
            battery_discharging = true;
        }
    }

    (has_ac && !ac_online) || battery_discharging
}
