//! Smoke test — run with `cargo run --example api_test`.
use std::fs::{self, File};
use std::io::Write;
use fast_reader::FastReader;

fn main() {
    let path = "/tmp/fr_smoke.txt";
    let mut f = File::create(path).unwrap();
    for i in 0..5 { writeln!(f, "line{i}").unwrap(); }
    drop(f);

    let mut r = FastReader::new(File::open(path).unwrap()).unwrap();
    // forward
    assert_eq!(r.next_line().unwrap().unwrap(), "line0");
    assert_eq!(r.next_line().unwrap().unwrap(), "line1");
    // backward
    assert_eq!(r.prev_line().unwrap().unwrap(), "line0");
    // bof/eof
    r.eof();
    assert_eq!(r.prev_line().unwrap().unwrap(), "line4");
    r.bof();
    assert_eq!(r.next_line().unwrap().unwrap(), "line0");

    // build_index + jump
    r.bof();
    r.build_index().unwrap();
    assert_eq!(r.line_count(), 5);
    assert_eq!(r.jump_to_line(3).unwrap().unwrap(), "line2");
    // next after jump
    assert_eq!(r.next_line().unwrap().unwrap(), "line3");
    // prev back
    assert_eq!(r.prev_line().unwrap().unwrap(), "line2");

    let _ = fs::remove_file(path);
    println!("ALL TESTS PASSED");
}
