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
    "if !Self::commit_frame_buffer(&self.queue, overlay, width, height) {",
    "let queue = self.queue.clone();\n        if !Self::commit_frame_buffer(&queue, overlay, width, height) {"
)
