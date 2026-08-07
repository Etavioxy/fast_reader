//! Rigorous index feature tests for fast_reader.
use std::fs::{self, File};
use std::io::Write;
use fast_reader::FastReader;

fn test_file(path: &str, lines: &[&str]) {
    let mut f = File::create(path).unwrap();
    for l in lines { writeln!(f, "{l}").unwrap(); }
}

#[test]
fn index_empty_file() {
    let p = "/tmp/fr_t_empty.txt";
    File::create(p).unwrap();
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.build_index().unwrap();
    assert_eq!(r.line_count(), 0);
    assert!(r.jump_to_line(1).unwrap().is_none());
    let _ = fs::remove_file(p);
}

#[test]
fn index_single_line() {
    let p = "/tmp/fr_t_single.txt";
    test_file(p, &["hello"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.build_index().unwrap();
    assert_eq!(r.line_count(), 1);
    assert_eq!(r.jump_to_line(1).unwrap().unwrap(), "hello");
    assert!(r.jump_to_line(0).unwrap().is_none());
    assert!(r.jump_to_line(2).unwrap().is_none());
    let _ = fs::remove_file(p);
}

#[test]
fn index_after_partial_read() {
    let p = "/tmp/fr_t_partial.txt";
    test_file(p, &["A", "B", "C", "D", "E"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.next_line().unwrap();
    r.next_line().unwrap(); // at "B" end
    r.build_index().unwrap();
    assert_eq!(r.line_count(), 5);
    assert_eq!(r.next_line().unwrap().unwrap(), "C"); // continues from position
    let _ = fs::remove_file(p);
}

#[test]
fn index_bof_jump_continue() {
    let p = "/tmp/fr_t_bof_jump.txt";
    test_file(p, &["A", "B", "C", "D", "E"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.bof();
    r.build_index().unwrap();
    assert_eq!(r.jump_to_line(4).unwrap().unwrap(), "D");
    assert_eq!(r.next_line().unwrap().unwrap(), "E");
    assert!(r.next_line().unwrap().is_none());
    assert_eq!(r.prev_line().unwrap().unwrap(), "D");
    let _ = fs::remove_file(p);
}

#[test]
fn index_eof_jump_back() {
    let p = "/tmp/fr_t_eof_jump.txt";
    test_file(p, &["A", "B", "C"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.eof();
    r.build_index().unwrap();
    assert_eq!(r.jump_to_line(1).unwrap().unwrap(), "A");
    assert_eq!(r.next_line().unwrap().unwrap(), "B");
    let _ = fs::remove_file(p);
}

#[test]
#[should_panic(expected = "build_index() must be called first")]
fn jump_without_index_panics() {
    let p = "/tmp/fr_t_no_idx.txt";
    test_file(p, &["x"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.jump_to_line(1).unwrap();
    let _ = fs::remove_file(p);
}

#[test]
fn random_line_with_index() {
    let p = "/tmp/fr_t_rand_idx.txt";
    test_file(p, &["A", "B", "C", "D", "E"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.build_index().unwrap();
    for _ in 0..50 {
        assert!(r.random_line().unwrap().is_some());
    }
    let _ = fs::remove_file(p);
}

#[test]
fn next_line_unaffected_by_index() {
    let p = "/tmp/fr_t_nl_idx.txt";
    test_file(p, &["A", "B", "C"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.build_index().unwrap();
    r.bof();
    assert_eq!(r.next_line().unwrap().unwrap(), "A");
    assert_eq!(r.next_line().unwrap().unwrap(), "B");
    assert_eq!(r.next_line().unwrap().unwrap(), "C");
    assert!(r.next_line().unwrap().is_none());
    let _ = fs::remove_file(p);
}

#[test]
fn prev_line_unaffected_by_index() {
    let p = "/tmp/fr_t_pl_idx.txt";
    test_file(p, &["A", "B", "C"]);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.build_index().unwrap();
    r.eof();
    assert_eq!(r.prev_line().unwrap().unwrap(), "C");
    assert_eq!(r.prev_line().unwrap().unwrap(), "B");
    assert_eq!(r.prev_line().unwrap().unwrap(), "A");
    assert!(r.prev_line().unwrap().is_none());
    let _ = fs::remove_file(p);
}

#[test]
fn crlf_index() {
    let p = "/tmp/fr_t_crlf.txt";
    let mut f = File::create(p).unwrap();
    f.write_all(b"A\r\nBB\r\nCCC\r\n").unwrap();
    drop(f);
    let mut r = FastReader::new(File::open(p).unwrap()).unwrap();
    r.build_index().unwrap();
    assert_eq!(r.line_count(), 3);
    assert_eq!(r.jump_to_line(2).unwrap().unwrap(), "BB");
    let _ = fs::remove_file(p);
}
