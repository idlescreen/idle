import sys

def replace_in_file(path, old, new):
    with open(path, "r") as f:
        content = f.read()
    if old not in content:
        print("OLD not found in content!")
    with open(path, "w") as f:
        f.write(content.replace(old, new))

replace_in_file(
    "idle/crates/wayland-present/src/presenter.rs",
    "pub struct OverlayPresenter {\n    command_tx: Sender<PresenterCommand>,",
    "pub struct OverlayPresenter {\n    command_tx: Sender<PresenterCommand>,\n    buffer_tx: Sender<Vec<u8>>,\n    buffer_rx: mpsc::Receiver<Vec<u8>>,"
)
replace_in_file(
    "idle/crates/wayland-present/src/presenter.rs",
    "let (command_tx, command_rx) = mpsc::channel();",
    "let (command_tx, command_rx) = mpsc::channel();\n        let (buffer_tx, buffer_rx) = mpsc::channel();"
)
replace_in_file(
    "idle/crates/wayland-present/src/presenter.rs",
    "command_tx,\n                visible,",
    "command_tx,\n                buffer_tx,\n                buffer_rx,\n                visible,"
)
replace_in_file(
    "idle/crates/wayland-present/src/presenter.rs",
    "pub fn submit_frame(&self, output_id: u32, width: u32, height: u32, pixels: Arc<Vec<u8>>) {\n        let _ = self.command_tx.send(PresenterCommand::UpdateFrame {\n            output_id,\n            width,\n            height,\n            pixels,\n        });\n    }",
    "pub fn submit_frame(&self, output_id: u32, width: u32, height: u32, pixels: Vec<u8>) {\n        let _ = self.command_tx.send(PresenterCommand::UpdateFrame {\n            output_id,\n            width,\n            height,\n            pixels,\n            return_pool: self.buffer_tx.clone(),\n        });\n    }\n\n    pub fn get_frame_buffer(&self, size: usize) -> Vec<u8> {\n        if let Ok(mut buf) = self.buffer_rx.try_recv() {\n            if buf.len() != size {\n                buf.resize(size, 0);\n            }\n            return buf;\n        }\n        vec![0; size]\n    }"
)

