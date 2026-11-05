#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
    // Exercise parsing, conversion, layout and destruction of retained DOMs.
    if input.len() <= patapim::MAX_HTML_BYTES {
        let _ = patapim::Document::from_local_html(input, 640, 480);
    }
});
