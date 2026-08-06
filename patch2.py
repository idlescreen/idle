import sys

def replace_in_file(path, old, new):
    with open(path, "r") as f:
        content = f.read()
    with open(path, "w") as f:
        f.write(content.replace(old, new))

replace_in_file(
    "idle/crates/wayland-present/src/overlay/thread.rs",
    "PresenterCommand::UpdateFrame {\n                output_id,\n                width,\n                height,\n                pixels,\n            } => state.update_frame(output_id, width, height, pixels),",
    "PresenterCommand::UpdateFrame {\n                output_id,\n                width,\n                height,\n                pixels,\n                return_pool,\n            } => {\n                state.update_frame(output_id, width, height, &pixels);\n                let _ = return_pool.send(pixels);\n            }"
)
