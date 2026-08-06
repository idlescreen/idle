import sys

def replace_in_file(path, old, new):
    with open(path, "r") as f:
        content = f.read()
    if old not in content:
        print("OLD not found in content!", old)
    with open(path, "w") as f:
        f.write(content.replace(old, new))

replace_in_file(
    "idle/crates/idle-ipc/src/lib.rs",
    "IpcResponse::FrameReady { scanlines: true }",
    "IpcResponse::FrameReady { scanlines: true, dirty: true }"
)
replace_in_file(
    "idle/crates/idle-ipc/src/lib.rs",
    "IpcResponse::FrameReady { scanlines: false }",
    "IpcResponse::FrameReady { scanlines: false, dirty: false }"
)
replace_in_file(
    "idle/crates/idle-ipc/src/lib.rs",
    "any::<bool>().prop_map(|scanlines| IpcResponse::FrameReady { scanlines }),",
    "(any::<bool>(), any::<bool>()).prop_map(|(scanlines, dirty)| IpcResponse::FrameReady { scanlines, dirty }),"
)

