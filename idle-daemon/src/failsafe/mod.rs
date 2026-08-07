// SPDX-License-Identifier: MIT

pub fn spawn_failsafe_locker() -> Result<(), String> {
    tracing::warn!("Plugin crashed! Executing fail-closed loginctl session lock...");

    // Asynchronously call loginctl to lock the session, so we don't block the daemon tick loop
    std::thread::spawn(|| {
        let status = std::process::Command::new("loginctl")
            .arg("lock-session")
            .status();

        match status {
            Ok(s) if s.success() => {
                tracing::info!("Successfully issued loginctl lock-session.");
            }
            Ok(s) => {
                tracing::error!(
                    "loginctl lock-session failed with status {s}. Executing swaylock fallback..."
                );
                let _ = std::process::Command::new("swaylock").arg("-f").status();
            }
            Err(e) => {
                tracing::error!("Failed to execute loginctl: {e}. Executing swaylock fallback...");
                let _ = std::process::Command::new("swaylock").arg("-f").status();
            }
        }
    });

    Ok(())
}
