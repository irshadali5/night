#![no_main]

use libfuzzer_sys::fuzz_target;
use night_core::ipc::{IpcCommand, IpcResponse};

fuzz_target!(|data: &[u8]| {
    let _ = postcard::from_bytes::<IpcCommand>(data);
    let _ = postcard::from_bytes::<IpcResponse>(data);
});
