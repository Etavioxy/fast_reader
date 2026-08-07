//! Benchmark harness: fast_reader vs std BufRead vs coreutils vs python
//!
//! Usage:
//!   run  <impl> <case> <data> [params...]  -> stdout result
//!   time <impl> <case> <data> [params...]  -> median wall ms
//!   all  <data>                            -> TSV matrix -> bench/data/results.tsv

use fast_reader::FastReader;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

const REPS: usize = 5;

// -----------------------------------------------------------------------
//  impl dispatch
// -----------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Impl { Fr, Std, Cli, Py }

impl Impl {
    fn name(self) -> &'static str {
        match self { Impl::Fr => "fr", Impl::Std => "std", Impl::Cli => "cli", Impl::Py => "py" }
    }
}

fn execute(imp: Impl, case: &str, data: &str, params: &[String]) -> io::Result<String> {
    match imp {
        Impl::Fr => run_fr(case, data, params),
        Impl::Std => run_std(case, data, params),
        Impl::Cli => run_cli(case, data, params),
        Impl::Py => run_py(case, data, params),
    }
}

fn p(params: &[String], i: usize) -> usize { params.get(i).map(|s| s.parse().unwrap_or(0)).unwrap_or(0) }

// -----------------------------------------------------------------------
//  fr — fast_reader lib
// -----------------------------------------------------------------------

fn run_fr(case: &str, data: &str, params: &[String]) -> io::Result<String> {
    let mut r = FastReader::new(File::open(data)?)?;
    match case {
        "head" => ok({
            let mut out = String::new();
            for _ in 0..p(params, 0) {
                if let Some(l) = r.next_line()? { out.push_str(&l); out.push('\n'); } else { break; }
            }
            out
        }),
        "tail" => ok({
            let n = p(params, 0);
            r.eof();
            let mut lines = Vec::new();
            for _ in 0..n { if let Some(l) = r.prev_line()? { lines.push(l); } else { break; } }
            lines.reverse();
            let mut out = String::new();
            for l in lines { out.push_str(&l); out.push('\n'); }
            out
        }),
        "line" => ok({
            let n = p(params, 0);
            let mut last = None;
            for _ in 0..n { match r.next_line()? { Some(l) => last = Some(l), None => break } }
            match last { Some(l) => format!("{l}\n"), None => String::new() }
        }),
        "range" => ok({
            let a = p(params, 0); let b = p(params, 1);
            let mut out = String::new();
            for _ in 1..a { if r.next_line()?.is_none() { return Ok(out); } }
            for _ in a..=b {
                if let Some(l) = r.next_line()? { out.push_str(&l); out.push('\n'); } else { break; }
            }
            out
        }),
        "sample" => ok({
            let k = p(params, 0);
            let mut out = String::new();
            for _ in 0..k {
                if let Some(l) = r.random_line()? { out.push_str(&l); out.push('\n'); }
            }
            out
        }),
        "reverse_line" => ok({
            let n = p(params, 0);
            r.eof();
            let mut target = None;
            for _ in 0..n { match r.prev_line()? { Some(l) => target = Some(l), None => break } }
            match target { Some(l) => format!("{l}\n"), None => String::new() }
        }),
        "count" => {
            let mut n: u64 = 0;
            while r.next_line()?.is_some() { n += 1; }
            Ok(n.to_string())
        }
        "parse" => {
            let mut ok: u64 = 0;
            while let Some(l) = r.next_line()? {
                if serde_json::from_str::<Value>(&l).is_ok() { ok += 1; }
            }
            Ok(ok.to_string())
        }
        "filter" => {
            let mut c: u64 = 0;
            while let Some(l) = r.next_line()? {
                if let Ok(v) = serde_json::from_str::<Value>(&l) {
                    if v["score"].as_f64().unwrap_or(0.0) > 500.0 { c += 1; }
                }
            }
            Ok(c.to_string())
        }
        "aggregate" => {
            let mut s: f64 = 0.0;
            while let Some(l) = r.next_line()? {
                if let Ok(v) = serde_json::from_str::<Value>(&l) {
                    s += v["score"].as_f64().unwrap_or(0.0);
                }
            }
            Ok(format!("{s:.2}"))
        }
        _ => panic!("unknown case: {case}"),
    }
}

fn ok(s: String) -> io::Result<String> { Ok(s) }

// -----------------------------------------------------------------------
//  std — BufRead
// -----------------------------------------------------------------------

fn run_std(case: &str, data: &str, params: &[String]) -> io::Result<String> {
    match case {
        "head" | "line" | "range" | "count" | "parse" | "filter" | "aggregate" => {
            std_seq(case, data, params)
        }
        "tail" | "reverse_line" => {
            let n = p(params, 0);
            let lines: Vec<String> = BufReader::new(File::open(data)?).lines().collect::<io::Result<_>>()?;
            if case == "tail" {
                Ok(lines[lines.len().saturating_sub(n)..].join("\n") + "\n")
            } else {
                Ok(match lines.get(lines.len().saturating_sub(n)) {
                    Some(l) => format!("{l}\n"),
                    None => String::new(),
                })
            }
        }
        "sample" => {
            let k = p(params, 0);
            let lines: Vec<String> = BufReader::new(File::open(data)?).lines().collect::<io::Result<_>>()?;
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hasher};
            let mut out = String::new();
            for i in 0..lines.len().min(k) {
                let h = RandomState::new().build_hasher().finish() as usize;
                out.push_str(&lines[(h + i * 7907) % lines.len()]);
                out.push('\n');
            }
            Ok(out)
        }
        _ => panic!("unknown case: {case}"),
    }
}

fn std_seq(case: &str, data: &str, params: &[String]) -> io::Result<String> {
    let mut reader = BufReader::new(File::open(data)?);
    let mut buf = String::new();
    match case {
        "head" => {
            let n = p(params, 0);
            let mut out = String::new();
            for _ in 0..n {
                buf.clear();
                if reader.read_line(&mut buf)? == 0 { break; }
                out.push_str(&buf);
            }
            Ok(out)
        }
        "line" => {
            let n = p(params, 0);
            let mut last = String::new();
            for _ in 0..n {
                buf.clear();
                if reader.read_line(&mut buf)? == 0 { break; }
                last = buf.clone();
            }
            Ok(last)
        }
        "range" => {
            let a = p(params, 0); let b = p(params, 1);
            let mut out = String::new();
            let mut i = 1usize;
            loop {
                buf.clear();
                if reader.read_line(&mut buf)? == 0 { break; }
                if i >= a && i <= b { out.push_str(&buf); }
                if i > b { break; }
                i += 1;
            }
            Ok(out)
        }
        "count" => {
            let mut n: u64 = 0;
            loop { buf.clear(); if reader.read_line(&mut buf)? == 0 { break; } n += 1; }
            Ok(n.to_string())
        }
        "parse" => {
            let mut ok: u64 = 0;
            loop {
                buf.clear();
                if reader.read_line(&mut buf)? == 0 { break; }
                if serde_json::from_str::<Value>(&buf).is_ok() { ok += 1; }
            }
            Ok(ok.to_string())
        }
        "filter" => {
            let mut c: u64 = 0;
            loop {
                buf.clear();
                if reader.read_line(&mut buf)? == 0 { break; }
                if let Ok(v) = serde_json::from_str::<Value>(&buf) {
                    if v["score"].as_f64().unwrap_or(0.0) > 500.0 { c += 1; }
                }
            }
            Ok(c.to_string())
        }
        "aggregate" => {
            let mut s: f64 = 0.0;
            loop {
                buf.clear();
                if reader.read_line(&mut buf)? == 0 { break; }
                if let Ok(v) = serde_json::from_str::<Value>(&buf) {
                    s += v["score"].as_f64().unwrap_or(0.0);
                }
            }
            Ok(format!("{s:.2}"))
        }
        _ => unreachable!(),
    }
}

// -----------------------------------------------------------------------
//  cli — coreutils
// -----------------------------------------------------------------------

fn spawn(args: &[String]) -> io::Result<String> {
    let out = Command::new(&args[0]).args(&args[1..]).output()?;
    if !out.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other,
            format!("{} failed: {}", args[0], String::from_utf8_lossy(&out.stderr))));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn run_cli(case: &str, data: &str, params: &[String]) -> io::Result<String> {
    match case {
        "head" => spawn(&[s("head"), s("-n"), s(&p(params,0).to_string()), s(data)]),
        "tail" => spawn(&[s("tail"), s("-n"), s(&p(params,0).to_string()), s(data)]),
        "line" => spawn(&[s("sed"), s("-n"), s(&format!("{}p", p(params,0))), s(data)]),
        "range" => spawn(&[s("sed"), s("-n"), s(&format!("{},{}p", p(params,0), p(params,1))), s(data)]),
        "sample" => spawn(&[s("shuf"), s("-n"), s(&p(params,0).to_string()), s(data)]),
        "reverse_line" => {
            let n = p(params, 0);
            spawn(&[s("sh"), s("-c"), s(&format!("t=$(wc -l < \"$1\"); sed -n \"$((t-{}))p\" \"$1\"", n-1)), s("sh"), s(data)])
        }
        "count" => Ok(spawn(&[s("wc"), s("-l"), s(data)])?.split_whitespace().next().unwrap_or("0").to_string()),
        "parse" => Ok(String::new()), // N/A
        "filter" => spawn(&[s("awk"), s("-f"), s("bench/scripts/filter.awk"), s(data)]),
        "aggregate" => spawn(&[s("awk"), s("-f"), s("bench/scripts/aggregate.awk"), s(data)]),
        _ => panic!("unknown case: {case}"),
    }
}

fn s(x: &str) -> String { x.to_string() }

// -----------------------------------------------------------------------
//  py — python reference
// -----------------------------------------------------------------------

fn run_py(case: &str, data: &str, params: &[String]) -> io::Result<String> {
    let out = Command::new("python")
        .arg("bench/scripts/py_runner.py").arg(case).arg(data)
        .args(params).output()?;
    if !out.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other,
            format!("python: {}", String::from_utf8_lossy(&out.stderr))));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// -----------------------------------------------------------------------
//  timing
// -----------------------------------------------------------------------

fn time_case(imp: Impl, case: &str, data: &str, params: &[String]) -> io::Result<f64> {
    execute(imp, case, data, params)?; // warmup
    let mut times: Vec<f64> = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let t0 = Instant::now();
        execute(imp, case, data, params)?;
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Ok(times[REPS / 2])
}

fn normalize(s: &str) -> String { s.replace("\r\n", "\n").trim().to_string() }

// -----------------------------------------------------------------------
//  all — full matrix
// -----------------------------------------------------------------------

/// Run a cell as a fresh subprocess — zero cross-impl state leakage.
fn time_standalone(imp: Impl, case: &str, data: &str, params: &[String]) -> io::Result<f64> {
    let exe = std::env::current_exe()?;
    let run = || -> io::Result<()> {
        let mut c = Command::new(&exe);
        c.arg("run").arg(imp.name()).arg(case).arg(data);
        for p in params { c.arg(p); }
        c.stdout(Stdio::null()).stderr(Stdio::null());
        c.status()?;
        Ok(())
    };
    run()?; // warmup
    let mut times = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let t0 = Instant::now();
        run()?;
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Ok(times[REPS / 2])
}

fn all(data: &str) -> io::Result<()> {
    let cases: Vec<(&str, Vec<String>)> = vec![
        ("head", vec!["10".into()]),
        ("tail", vec!["10".into()]),
        ("line", vec!["500000".into()]),
        ("range", vec!["500000".into(), "500009".into()]),
        ("sample", vec!["100".into()]),
        ("reverse_line", vec!["5000".into()]),
        ("count", vec![]),
        ("parse", vec![]),
        ("filter", vec![]),
        ("aggregate", vec![]),
    ];
    let impls = [Impl::Fr, Impl::Std, Impl::Cli, Impl::Py];

    // golden via python
    let mut golden: HashMap<&str, String> = HashMap::new();
    for (case, params) in &cases {
        if *case == "sample" { continue; }
        golden.insert(case, execute(Impl::Py, case, data, params)?);
    }
    let valid: HashSet<String> =
        BufReader::new(File::open(data)?).lines().collect::<io::Result<_>>()?;

    let mut out = String::from("case\timpl\tmedian_ms\tstatus\n");
    for (case, params) in &cases {
        for &imp in &impls {
            let (median, status) = if imp == Impl::Cli && *case == "parse" {
                (0.0, "N/A")
            } else {
                let res = execute(imp, case, data, params)?;
                let ok = if *case == "sample" {
                    let k = p(params, 0);
                    let members: Vec<&str> = res.split('\n').filter(|m| !m.is_empty())
                        .map(|m| m.trim_end_matches('\r')).collect();
                    members.len() == k && members.iter().all(|m| valid.contains(*m))
                } else {
                    normalize(&res) == normalize(golden.get(*case).unwrap())
                };
                if ok { (time_standalone(imp, case, data, params)?, "PASS") }
                else { (0.0, "FAIL") }
            };
            out.push_str(&format!("{case}\t{}\t{median:.3}\t{status}\n", imp.name()));
            eprintln!("{case:<14} {:<4} time={median:.1}ms {status}", imp.name());
        }
    }
    std::fs::write("bench/data/results.tsv", out)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} (run|time|all) [impl case data params...]", args[0]);
        std::process::exit(2);
    }
    match args[1].as_str() {
        "run" => {
            let imp = match args[2].as_str() { "fr"=>Impl::Fr, "std"=>Impl::Std, "cli"=>Impl::Cli, "py"=>Impl::Py, _=>panic!() };
            print!("{}", execute(imp, &args[3], &args[4], &args[5..])?);
        }
        "time" => {
            let imp = match args[2].as_str() { "fr"=>Impl::Fr, "std"=>Impl::Std, "cli"=>Impl::Cli, "py"=>Impl::Py, _=>panic!() };
            println!("{:.2}", time_case(imp, &args[3], &args[4], &args[5..])?);
        }
        "all" => all(&args[2])?,
        _ => panic!("unknown mode"),
    }
    Ok(())
}
