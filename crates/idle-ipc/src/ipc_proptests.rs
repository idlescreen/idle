// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen
//
// Split out of `lib.rs` to keep that file under the 256-line cap (F-016/PROBE F-006).
// Referenced from `lib.rs` via `#[path = "ipc_proptests.rs"]`.

use super::*;
use proptest::prelude::*;

fn arb_command() -> impl Strategy<Value = IpcCommand> {
    prop_oneof![
        (1u32..=512, 1u32..=512).prop_map(|(cols, rows)| IpcCommand::Init { cols, rows }),
        any::<u64>().prop_map(|dt_micros| IpcCommand::TickAndDraw { dt_micros }),
        any::<f32>().prop_filter_map("finite hz", |hz| {
            hz.is_finite()
                .then_some(IpcCommand::SetSimulationRate { hz })
        }),
        Just(IpcCommand::Stop),
    ]
}

fn arb_response() -> impl Strategy<Value = IpcResponse> {
    prop_oneof![
        Just(IpcResponse::Ready),
        (any::<bool>(), any::<bool>())
            .prop_map(|(scanlines, dirty)| IpcResponse::FrameReady { scanlines, dirty }),
        Just(IpcResponse::Ack),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every command encodes and decodes to an equal value.
    #[test]
    fn command_roundtrip(cmd in arb_command()) {
        let mut buf = Vec::new();
        cmd.write_to(&mut buf).expect("write");
        let decoded = IpcCommand::read_from(&buf[..]).expect("read");
        prop_assert_eq!(cmd, decoded);
    }

    /// Every response encodes and decodes to an equal value.
    #[test]
    fn response_roundtrip(resp in arb_response()) {
        let mut buf = Vec::new();
        resp.write_to(&mut buf).expect("write");
        let decoded = IpcResponse::read_from(&buf[..]).expect("read");
        prop_assert_eq!(resp, decoded);
    }

    /// SHM size is at least the header and grows linearly with cells.
    #[test]
    fn shm_size_monotonic(cols in 0usize..512, rows in 0usize..512) {
        let size = compute_shm_size(cols, rows).expect("no overflow in range");
        let header = std::mem::size_of::<SharedMemoryHeader>();
        let cell = std::mem::size_of::<FfiTerminalCell>();
        prop_assert!(size >= header);
        prop_assert_eq!(size, header + cols * rows * cell);
        if cols > 0 && rows > 0 {
            let smaller = compute_shm_size(cols - 1, rows).expect("smaller");
            prop_assert!(smaller < size || cols == 1);
        }
    }

    /// Adversarial dims either validate cleanly or are rejected; size never panics.
    #[test]
    fn adversarial_dims_never_panic(cols in any::<u32>(), rows in any::<u32>()) {
        let c = cols as usize;
        let r = rows as usize;
        let _ = validate_grid_dims(c, r);
        let _ = compute_shm_size(c, r);
    }

    /// Unknown command tags are rejected.
    #[test]
    fn invalid_command_tags_fail(tag in 4u8..=255) {
        prop_assert!(IpcCommand::read_from(&[tag][..]).is_err());
    }

    /// Unknown response tags are rejected.
    #[test]
    fn invalid_response_tags_fail(tag in 3u8..=255) {
        prop_assert!(IpcResponse::read_from(&[tag][..]).is_err());
    }
}
