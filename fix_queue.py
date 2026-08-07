import os
import re

def repl(match):
    prefix = match.group(1)
    call = match.group(2)
    return f"{prefix}try_send({call}).map_err(|_| zbus::fdo::Error::LimitsExceeded(\"Command queue full\".into()))?;"

# idle-daemon/src/controller/mod.rs
path = "idle-daemon/src/controller/mod.rs"
with open(path, "r") as f:
    content = f.read()
content = content.replace("mpsc::Sender<DaemonCommand>", "mpsc::SyncSender<DaemonCommand>")
content = content.replace("mpsc::channel()", "mpsc::sync_channel(16)")
with open(path, "w") as f:
    f.write(content)

# idle-daemon/src/dbus_server/service.rs
path = "idle-daemon/src/dbus_server/service.rs"
with open(path, "r") as f:
    content = f.read()

content = re.sub(r'(let _ = self\s*\.controller\s*\.command_tx\s*)\.send\((.*?)\);', repl, content)
with open(path, "w") as f:
    f.write(content)

# idle-daemon/src/dbus_server/screensaver.rs
path = "idle-daemon/src/dbus_server/screensaver.rs"
with open(path, "r") as f:
    content = f.read()

def repl_screen(match):
    prefix = match.group(1)
    call = match.group(2)
    if "simulate_user_activity" in content[:match.start()] or "lock(" in content[:match.start()]:
        # inside simulate_user_activity or lock which return ()
        pass # wait, it's easier to just do it manually
    return f"{prefix}try_send({call}).map_err(|_| zbus::fdo::Error::LimitsExceeded(\"Command queue full\".into()))?;"

# let's just use string replace for screensaver to be safe
content = content.replace("""        let _ = self
            .controller
            .command_tx
            .send(DaemonCommand::StopPresentation);""", """        self.controller
            .command_tx
            .try_send(DaemonCommand::StopPresentation)
            .unwrap_or_else(|_| tracing::warn!("Command queue full"));""")

content = content.replace("""            let _ = self
                .controller
                .command_tx
                .send(DaemonCommand::Preview(saver));""", """            self.controller
                .command_tx
                .try_send(DaemonCommand::Preview(saver))
                .map_err(|_| zbus::fdo::Error::LimitsExceeded("Command queue full".into()))?;""")

content = content.replace("""            let _ = self
                .controller
                .command_tx
                .send(DaemonCommand::StopPresentation);""", """            self.controller
                .command_tx
                .try_send(DaemonCommand::StopPresentation)
                .map_err(|_| zbus::fdo::Error::LimitsExceeded("Command queue full".into()))?;""")

# fix lock unwrap
content = content.replace(".unwrap().presentation_active", ".unwrap_or_else(|e| e.into_inner()).presentation_active")

with open(path, "w") as f:
    f.write(content)

# tests
path = "idle-daemon/src/controller/commands_tests.rs"
with open(path, "r") as f:
    content = f.read()
content = content.replace(".send(", ".try_send(")
with open(path, "w") as f:
    f.write(content)

