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
            Ok(None) => { /* small file, bad offset — retry */ }
            Err(e) => panic!("random_line error: {e}"),
        }
    }

    // === indexed scenario: read halfway → jump 70% → continue ===
    r.bof();
    r.build_index().unwrap();
    assert_eq!(r.line_count(), 5);

    // read forward to middle (buffer-based — index NOT used)
    r.bof();
    for _ in 0..2 { r.next_line().unwrap(); }
    assert_eq!(r.current_line().unwrap().unwrap(), "B B BB BBB");

    // jump to line 4 (indexed)
    let jumped = r.jump_to_line(4).unwrap().unwrap();
    assert_eq!(jumped, "DDDD  DDDDD DD DDD DDD DD");

    // next_line from jumped position (buffer-based, works seamlessly)
    let nxt = r.next_line().unwrap().unwrap();
    assert_eq!(nxt, "EEEE  EEEEE  EEEE  EEEEE");

    // prev_line back to line 4
    let prv = r.prev_line().unwrap().unwrap();
    assert_eq!(prv, jumped);

    // random_line with index — perfect uniform distribution
    for _ in 0..10 {
        assert!(r.random_line().unwrap().is_some());
    }

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
