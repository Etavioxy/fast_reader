use std::fs::{self, File};
use std::io::Write;
use fast_reader::FastReader;

fn main() {
    // build a test file
    let path = "/tmp/fr_test.txt";
    let mut f = File::create(path).unwrap();
    writeln!(f, "AAAA AAAA").unwrap();
    writeln!(f, "B B BB BBB").unwrap();
    writeln!(f, "CCCC  CCCCC").unwrap();
    writeln!(f, "DDDD  DDDDD DD DDD DDD DD").unwrap();
    writeln!(f, "EEEE  EEEEE  EEEE  EEEEE").unwrap();
    drop(f);

    let file = File::open(path).unwrap();
    let mut r = FastReader::new(file).unwrap();

    // === forward ===
    assert_eq!(r.next_line().unwrap().unwrap(), "AAAA AAAA", "L1 fwd");
    assert_eq!(r.next_line().unwrap().unwrap(), "B B BB BBB", "L2 fwd");
    assert_eq!(r.next_line().unwrap().unwrap(), "CCCC  CCCCC", "L3 fwd");
    assert_eq!(r.current_line().unwrap().unwrap(), "CCCC  CCCCC", "current L3");

    // === backward ===
    assert_eq!(r.prev_line().unwrap().unwrap(), "B B BB BBB", "L2 back");
    assert_eq!(r.prev_line().unwrap().unwrap(), "AAAA AAAA", "L1 back");
    assert!(r.prev_line().unwrap().is_none(), "before BOF = None");

    // === bof / eof ===
    r.bof();
    assert_eq!(r.next_line().unwrap().unwrap(), "AAAA AAAA", "bof->L1");
    r.eof();
    assert_eq!(r.prev_line().unwrap().unwrap(), "EEEE  EEEEE  EEEE  EEEEE", "eof->last");
    assert_eq!(r.prev_line().unwrap().unwrap(), "DDDD  DDDDD DD DDD DDD DD", "eof->L4");

    // === random_line ===
    for _ in 0..20 {
        match r.random_line() {
            Ok(Some(l)) => assert!(!l.is_empty()),
            Ok(None) => { /* small file, bad offset — retry handled by lib */ }
            Err(e) => panic!("random_line error: {e}"),
        }
    }

    // === build_index + jump_to_line ===
    r.bof();
    r.build_index().unwrap();
    assert_eq!(r.line_count(), 5, "5 lines");

    // after build_index(), next_line still works (index doesn't change it)
    r.bof();
    assert_eq!(r.next_line().unwrap().unwrap(), "AAAA AAAA", "post-index fwd");

    // jump_to_line
    assert_eq!(r.jump_to_line(3).unwrap().unwrap(), "CCCC  CCCCC", "jump L3");
    assert_eq!(r.jump_to_line(5).unwrap().unwrap(), "EEEE  EEEEE  EEEE  EEEEE", "jump L5");
    assert!(r.jump_to_line(0).unwrap().is_none(), "jump L0");
    assert!(r.jump_to_line(99).unwrap().is_none(), "jump L99 out of range");

    // === empty file ===
    let empty_path = "/tmp/fr_empty_test.txt";
    File::create(empty_path).unwrap();
    let mut r2 = FastReader::new(File::open(empty_path).unwrap()).unwrap();
    assert!(r2.next_line().unwrap().is_none(), "empty file next = None");
    assert_eq!(r2.file_size(), 0, "empty file size");

    // cleanup
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(empty_path);

    println!("ALL TESTS PASSED");
}
