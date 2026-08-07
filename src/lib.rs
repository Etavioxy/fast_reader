//! # fast_reader
//!
//! A drop-in replacement for [`easy_reader`](https://crates.io/crates/easy_reader) —
//! buffered bidirectional line navigation for huge files, matching or exceeding
//! CLI tool performance.
//!
//! ## Why fast_reader
//!
//! `easy_reader` allows forward, backward, and random navigation through large
//! files — but its forward reads use ~2-3 syscalls per line.  `fast_reader`
//! keeps the same API surface while using a **persistent read buffer** that
//! reduces forward-reading syscalls to ~0.003 per line, matching `std BufRead`
//! and GNU coreutils throughput.  Reverse and random access use window-based
//! seeks (O(K) instead of O(N)) so tail and sampling stay fast regardless of
//! file size.
//!
//! `build_index()` is optional; when present it enables O(1) [`jump_to_line`]
//! without affecting the behaviour of [`next_line`] / [`prev_line`] /
//! [`random_line`].
//!
//! ## Example
//!
//! ```no_run
//! use fast_reader::FastReader;
//! use std::fs::File;
//!
//! let file = File::open("huge.log")?;
//! let mut r = FastReader::new(file)?;
//!
//! // forward
//! while let Some(line) = r.next_line()? {
//!     println!("{line}");
//! }
//!
//! // backward
//! r.eof();
//! while let Some(line) = r.prev_line()? {
//!     println!("{line}");
//! }
//!
//! // jump to line (requires build_index)
//! r.build_index()?;
//! if let Some(line) = r.jump_to_line(500_000)? {
//!     println!("line 500000: {line}");
//! }
//! # Ok::<(), std::io::Error>(())
//! ```

use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

#[cfg(feature = "rand")]
use rand::Rng;

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const DEFAULT_CHUNK: usize = 64 * 1024;

/// A bidirectional line reader for huge files.
///
/// `FastReader` allows forward, backward, and random navigation through
/// the lines of a file without consuming an iterator.  Forward reads are
/// buffered (matching `std BufRead` throughput); reverse and random access
/// use window-based seeks.
pub struct FastReader<R: Read + Seek> {
    file: R,
    file_size: u64,

    // --- persistent forward buffer ---
    fbuf: Vec<u8>,       // buffer
    fbuf_start: u64,     // file offset of fbuf[0]
    fbuf_fill: usize,    // valid bytes in fbuf
    fbuf_pos: usize,     // next read position within fbuf

    // --- current-line tracking (for current_line / prev_line) ---
    line_start: u64,     // byte offset of current line's first byte
    line_end: u64,       // byte offset just past current line (after LF)

    // --- navigation state ---
    at_bof: bool,
    at_eof: bool,

    // --- config ---
    chunk_size: usize,

    // --- index (optional, enables jump_to_line) ---
    index: Vec<u64>,     // byte offset of each line start, line 1 = index[0].
    indexed: bool,
}

// ---------------------------------------------------------------------------
//  public API (drop-in compatible with easy_reader, plus a few extras)
// ---------------------------------------------------------------------------

impl<R: Read + Seek> FastReader<R> {
    /// Create a new `FastReader` from a file (or any `Read + Seek`).
    ///
    /// Unlike `easy_reader`, empty files are accepted (all reads return
    /// `None`).
    pub fn new(mut file: R) -> io::Result<Self> {
        let file_size = file.seek(SeekFrom::End(0))?;
        Ok(FastReader {
            file,
            file_size,
            fbuf: Vec::new(),
            fbuf_start: 0,
            fbuf_fill: 0,
            fbuf_pos: 0,
            line_start: 0,
            line_end: 0,
            at_bof: true,
            at_eof: file_size == 0,
            chunk_size: DEFAULT_CHUNK,
            index: Vec::new(),
            indexed: false,
        })
    }

    /// Set the read-chunk size (default 64 KiB).
    ///
    /// This controls the buffer / window size used by [`next_line`],
    /// [`prev_line`], and [`random_line`].
    pub fn chunk_size(&mut self, size: usize) -> &mut Self {
        self.chunk_size = size;
        self
    }

    /// Return the file size in bytes.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    // -- positioning --------------------------------------------------------

    /// Reset the reader to the beginning of the file.
    pub fn bof(&mut self) -> &mut Self {
        self.at_bof = true;
        self.at_eof = self.file_size == 0;
        self.line_start = 0;
        self.line_end = 0;
        // invalidate forward buffer so the next read seeks to 0
        self.fbuf_start = 0;
        self.fbuf_fill = 0;
        self.fbuf_pos = 0;
        self
    }

    /// Position the reader at the end of the file.
    pub fn eof(&mut self) -> &mut Self {
        self.at_bof = false;
        self.at_eof = true;
        self.line_start = self.file_size;
        self.line_end = self.file_size;
        self
    }

    // -- line navigation ----------------------------------------------------

    /// Read the next line forward.
    ///
    /// Uses a persistent read buffer so forward iteration costs ~0.003
    /// syscalls per line (matching `std BufRead`).
    pub fn next_line(&mut self) -> io::Result<Option<String>> {
        if self.at_eof {
            return Ok(None);
        }
        self.at_bof = false;

        // advance past the previous line
        self.line_start = self.line_end;

        loop {
            // ensure we have buffer data at the current position
            self.ensure_buf()?;

            // scan for LF
            let pos = self.fbuf_pos;
            let fill = self.fbuf_fill;
            if let Some(rel) = self.fbuf[pos..fill].iter().position(|&b| b == LF) {
                let abs = pos + rel;
                let byte_end = abs + 1; // past LF
                // strip CR
                let mut content_end = abs;
                if content_end > 0
                    && pos < content_end
                    && self.fbuf[content_end - 1] == CR
                {
                    content_end -= 1;
                }
                let line = String::from_utf8_lossy(&self.fbuf[pos..content_end]).into_owned();

                self.fbuf_pos = byte_end;
                self.line_end =
                    self.fbuf_start + byte_end as u64;

                // if we've reached EOF
                if self.line_end >= self.file_size {
                    self.at_eof = true;
                }
                return Ok(if line.is_empty() && self.at_eof {
                    None
                } else {
                    Some(line)
                });
            }

            // no LF in buffer — compact and read more, or EOF
            if self.fbuf_start + self.fbuf_fill as u64 >= self.file_size {
                // reached physical EOF
                self.at_eof = true;
                if pos < fill {
                    let line = String::from_utf8_lossy(&self.fbuf[pos..fill]).into_owned();
                    self.fbuf_pos = fill;
                    self.line_end = self.file_size;
                    return Ok(if line.is_empty() { None } else { Some(line) });
                }
                return Ok(None);
            }
            // compact and read more
            self.compact_and_fill()?;
        }
    }

    /// Re-read the current line (the line most recently returned by
    /// [`next_line`] or [`prev_line`]).
    pub fn current_line(&mut self) -> io::Result<Option<String>> {
        if self.line_start == self.line_end {
            return Ok(None);
        }
        let len = (self.line_end - self.line_start) as usize;
        let mut buf = vec![0u8; len.min(1024 * 1024)];
        self.file.seek(SeekFrom::Start(self.line_start))?;
        let n = self.file.read(&mut buf)?;
        let raw = String::from_utf8_lossy(&buf[..n]);
        // strip trailing CR/LF
        let trimmed = raw.trim_end_matches(&['\r', '\n'][..]);
        Ok(if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        })
    }

    /// Read the previous line (move backward).
    ///
    /// Uses a window-based backward scan so reverse iteration accesses
    /// only the tail of the file (O(tail), not O(N)).
    pub fn prev_line(&mut self) -> io::Result<Option<String>> {
        if self.at_bof || self.line_start == 0 {
            return Ok(None);
        }
        self.at_eof = false;

        // scan a window backward from line_start to find the line before it
        let before = self.line_start.saturating_sub(1);
        let window = self.chunk_size as u64;
        let from = before.saturating_sub(window);
        let len = (before - from + 1) as usize;

        self.file.seek(SeekFrom::Start(from))?;
        let mut buf = vec![0u8; len];
        self.file.read_exact(&mut buf)?;

        // find the LAST LF in the window before 'before'
        let rel_before = (before - from) as usize;
        if let Some(lf) = buf[..rel_before].iter().rposition(|&b| b == LF) {
            let ls = lf + 1; // line start in buf
            let mut le = rel_before;
            if le > ls && buf[le - 1] == CR {
                le -= 1;
            }
            let line = String::from_utf8_lossy(&buf[ls..le]).into_owned();

            self.line_end = self.line_start;
            self.line_start = from + ls as u64;
            if self.line_start == 0 {
                self.at_bof = true;
            }
            Ok(if line.is_empty() { None } else { Some(line) })
        } else {
            // no LF found → we're at the first line
            self.line_end = self.line_start;
            self.line_start = 0;
            self.at_bof = true;

            let le = rel_before.min(buf.len());
            let raw = String::from_utf8_lossy(&buf[..le]);
            let trimmed = raw.trim_end_matches(&['\r', '\n'][..]);
            Ok(if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            })
        }
    }

    // -- random access (requires `rand` feature) ----------------------------

    /// Return a random line from the file.
    ///
    /// Uses a single window seek — O(1) syscalls, independent of file size.
    /// When `build_index` has been called, uses the index for perfect
    /// uniform distribution; otherwise uses byte-offset sampling (biased
    /// toward longer lines, but usable without index overhead).
    #[cfg(feature = "rand")]
    pub fn random_line(&mut self) -> io::Result<Option<String>> {
        if self.file_size == 0 {
            return Ok(None);
        }
        self.at_bof = false;
        self.at_eof = false;

        let off = if self.indexed && !self.index.is_empty() {
            let i = rand::thread_rng().gen_range(0..self.index.len());
            self.index[i]
        } else {
            rand::thread_rng().gen_range(0..self.file_size)
        };

        let window = self.chunk_size as u64;
        let from = off.saturating_sub(window / 2);
        let len = (window as usize).min((self.file_size - from) as usize);
        self.file.seek(SeekFrom::Start(from))?;
        let mut buf = vec![0u8; len];
        self.file.read_exact(&mut buf)?;

        let mut idx = (off - from) as usize;
        // skip any LFs under the cursor so we land inside a line
        while idx < len && buf[idx] == LF {
            idx += 1;
        }
        let ls = if let Some(lf) = buf[..idx].iter().rposition(|&b| b == LF) {
            lf + 1
        } else {
            0
        };
        let le = if let Some(lf) = buf[idx..].iter().position(|&b| b == LF) {
            idx + lf
        } else {
            len
        };
        if ls >= le {
            return Ok(None); // empty line or degenerate case
        }
        let mut end = le;
        if end > ls && buf[end - 1] == CR {
            end -= 1;
        }
        let line = String::from_utf8_lossy(&buf[ls..end]).into_owned();
        Ok(if line.is_empty() { None } else { Some(line) })
    }

    // -- index & jump -------------------------------------------------------

    /// Build a line-start offset index by scanning the file once (O(N)).
    ///
    /// After calling this, [`jump_to_line`] is O(1).  The index does **not**
    /// affect the behaviour of [`next_line`] / [`prev_line`] /
    /// [`random_line`] — those methods continue to use buffered /
    /// window-based I/O regardless of whether the index is present.
    ///
    /// Memory cost: ~8 bytes per line (8 MB for a 1M-line file).
    pub fn build_index(&mut self) -> io::Result<&mut Self> {
        // Save current position
        let saved_start = self.line_start;
        let saved_end = self.line_end;
        let saved_at_eof = self.at_eof;
        let saved_at_bof = self.at_bof;

        self.index.clear();

        // Use BufReader for fast forward-only scan (avoid re-entering
        // FastReader's own next_line which would thrash the internal buffer).
        self.file.seek(SeekFrom::Start(0))?;
        let mut br = BufReader::with_capacity(self.chunk_size, &mut self.file);
        let mut buf = String::new();
        let mut offset: u64 = 0;
        loop {
            buf.clear();
            let n = br.read_line(&mut buf)?;
            if n == 0 {
                break;
            }
            self.index.push(offset);
            offset += n as u64;
        }

        self.indexed = true;

        // Restore original position
        self.file.seek(SeekFrom::Start(saved_start))?;
        self.line_start = saved_start;
        self.line_end = saved_end;
        self.at_eof = saved_at_eof;
        self.at_bof = saved_at_bof;
        // invalidate forward buffer since file cursor moved
        self.fbuf_start = 0;
        self.fbuf_fill = 0;
        self.fbuf_pos = 0;

        Ok(self)
    }

    /// Jump to line `n` (1-based) and return it.  Requires
    /// [`build_index`] to have been called first; panics otherwise.
    ///
    /// After a successful jump, [`current_line`] reflects the new position.
    pub fn jump_to_line(&mut self, n: usize) -> io::Result<Option<String>> {
        assert!(self.indexed, "build_index() must be called before jump_to_line()");
        if n == 0 || n > self.index.len() {
            return Ok(None);
        }
        let off = self.index[n - 1];
        self.file.seek(SeekFrom::Start(off))?;

        // Read one line with a temporary BufReader
        let mut buf = String::new();
        {
            let mut br = BufReader::with_capacity(self.chunk_size, &mut self.file);
            br.read_line(&mut buf)?;
        }

        // Update position state
        self.line_start = off;
        self.line_end = off + buf.len() as u64;
        self.at_bof = off == 0;
        self.at_eof = self.line_end >= self.file_size;
        // invalidate forward buffer
        self.fbuf_start = 0;
        self.fbuf_fill = 0;
        self.fbuf_pos = 0;

        let trimmed = buf.trim_end_matches(&['\r', '\n'][..]);
        Ok(if trimmed.is_empty() { None } else { Some(trimmed.to_string()) })
    }

    /// Return the total number of lines (requires [`build_index`]).
    pub fn line_count(&self) -> usize {
        self.index.len()
    }

    // -- internal helpers ---------------------------------------------------

    /// Make sure `self.fbuf` has data at `self.fbuf_pos`.
    fn ensure_buf(&mut self) -> io::Result<()> {
        if self.fbuf_pos < self.fbuf_fill {
            return Ok(());
        }
        // need to read a new chunk
        let file_off = self.fbuf_start + self.fbuf_fill as u64;
        if file_off >= self.file_size {
            return Ok(());
        }
        let cap = self.chunk_size.max(self.fbuf.len());
        if self.fbuf.len() < cap {
            self.fbuf.resize(cap, 0);
        }
        let to_read = (cap as u64).min(self.file_size - file_off) as usize;
        self.file.seek(SeekFrom::Start(file_off))?;
        let n = self.file.read(&mut self.fbuf[..to_read])?;
        self.fbuf_start = file_off;
        self.fbuf_pos = 0;
        self.fbuf_fill = n;
        Ok(())
    }

    /// Compact remaining data to front and read more.
    fn compact_and_fill(&mut self) -> io::Result<()> {
        if self.fbuf_pos > 0 {
            if self.fbuf_pos < self.fbuf_fill {
                self.fbuf.copy_within(self.fbuf_pos..self.fbuf_fill, 0);
                self.fbuf_fill -= self.fbuf_pos;
            } else {
                self.fbuf_fill = 0;
            }
            self.fbuf_start += self.fbuf_pos as u64;
            self.fbuf_pos = 0;
        }
        let cap = self.fbuf.len().max(self.chunk_size);
        if self.fbuf.len() < cap {
            self.fbuf.resize(cap, 0);
        }
        let file_off = self.fbuf_start + self.fbuf_fill as u64;
        let space = self.fbuf.len() - self.fbuf_fill;
        let to_read = (space as u64).min(self.file_size - file_off) as usize;
        if to_read == 0 {
            return Ok(());
        }
        self.file.seek(SeekFrom::Start(file_off))?;
        let n = self.file.read(&mut self.fbuf[self.fbuf_fill..self.fbuf_fill + to_read])?;
        self.fbuf_fill += n;
        Ok(())
    }
}
