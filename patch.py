import sys

def replace_in_file(path, old, new):
    with open(path, "r") as f:
        content = f.read()
    with open(path, "w") as f:
        f.write(content.replace(old, new))

replace_in_file(
    "idle/crates/wayland-present/src/overlay/thread.rs",
    "pub enum PresenterCommand {\n    ShowSolid(OverlayAppearance),\n    ShowScreensaver,\n    UpdateFrame {\n        output_id: u32,\n        width: u32,\n        height: u32,\n        pixels: Arc<Vec<u8>>,\n    },\n    Hide,\n}",
    "pub enum PresenterCommand {\n    ShowSolid(OverlayAppearance),\n    ShowScreensaver,\n    UpdateFrame {\n        output_id: u32,\n        width: u32,\n        height: u32,\n        pixels: Vec<u8>,\n        return_pool: Sender<Vec<u8>>,\n    },\n    Hide,\n}"
)
