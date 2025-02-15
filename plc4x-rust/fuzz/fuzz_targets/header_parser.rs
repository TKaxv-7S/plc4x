#![no_main]
use libfuzzer_sys::fuzz_target;
use plc4x_rust::s7::S7Header;

fuzz_target!(|data: &[u8]| {
    let _ = S7Header::parse(data);
});
