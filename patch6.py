import sys

def replace_in_file(path, old, new):
    with open(path, "r") as f:
        content = f.read()
    if old not in content:
        print("OLD not found in content!")
    with open(path, "w") as f:
        f.write(content.replace(old, new))

replace_in_file(
    "idle/idle-daemon/src/presentation/ipc_session.rs",
    "pixel_buf: std::sync::Arc<Vec<u8>>,\n",
    ""
)

replace_in_file(
    "idle/idle-daemon/src/presentation/ipc_session.rs",
    "pixel_buf: std::sync::Arc::new(Vec::new()),\n",
    ""
)

replace_in_file(
    "idle/idle-daemon/src/presentation/ipc_session.rs",
    "pub fn raster_viewport(\n        &mut self,\n        col_start: usize,\n        row_start: usize,\n        cols: usize,\n        rows: usize,\n        grid_cols: usize,\n        _grid_rows: usize,\n        width: u32,\n        height: u32,\n        scanlines: bool,\n    ) -> std::sync::Arc<Vec<u8>> {",
    "pub fn raster_viewport(\n        &mut self,\n        col_start: usize,\n        row_start: usize,\n        cols: usize,\n        rows: usize,\n        grid_cols: usize,\n        _grid_rows: usize,\n        width: u32,\n        height: u32,\n        scanlines: bool,\n        pixel_buf: &mut Vec<u8>,\n    ) {"
)

replace_in_file(
    "idle/idle-daemon/src/presentation/ipc_session.rs",
    "std::sync::Arc::make_mut(&mut self.pixel_buf),\n            col_start,\n            row_start,\n            cols,\n            rows,\n            grid_cols,\n            width,\n            height,\n            scanlines,\n        );\n        self.pixel_buf.clone()\n    }",
    "pixel_buf,\n            col_start,\n            row_start,\n            cols,\n            rows,\n            grid_cols,\n            width,\n            height,\n            scanlines,\n        );\n    }"
)

