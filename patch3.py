import sys

def replace_in_file(path, old, new):
    with open(path, "r") as f:
        content = f.read()
    if old not in content:
        print("OLD not found in content!")
    with open(path, "w") as f:
        f.write(content.replace(old, new))

replace_in_file(
    "idle/crates/wayland-present/src/overlay/state/overlay.rs",
    "pub fn update_frame(\n        &mut self,\n        output_id: u32,\n        width: u32,\n        height: u32,\n        pixels: std::sync::Arc<Vec<u8>>,\n    ) {",
    "pub fn update_frame(\n        &mut self,\n        output_id: u32,\n        width: u32,\n        height: u32,\n        pixels: &[u8],\n    ) {"
)
replace_in_file(
    "idle/crates/wayland-present/src/overlay/state/overlay.rs",
    "&self.queue,\n            width,\n            height,\n            &pixels,\n        )",
    "&self.queue,\n            width,\n            height,\n            pixels,\n        )"
)
