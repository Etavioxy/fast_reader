#!/usr/bin/env python3
"""Generate REPORT.md from benchmark TSV."""
import argparse

IMPLS = ["fr", "std", "cli", "py"]
CASES = [
    ("head", "开头 10 行"), ("tail", "末尾 10 行"),
    ("line", "第 500000 行"), ("range", "第 500000~500009 行"),
    ("sample", "随机 100 行"), ("reverse_line", "倒数第 5000 行"),
    ("count", "全文行数"), ("parse", "逐行 JSON 解析"),
    ("filter", "过滤 score>500"), ("aggregate", "score 求和"),
]


def load(tsv):
    rows = {}
    with open(tsv, encoding="utf-8") as f:
        for line in f:
            p = line.strip().split("\t")
            if len(p) == 4 and p[0] != "case":
                rows[(p[0], p[1])] = (float(p[2]), p[3])
    return rows


def fmt(ms):
    if ms >= 1000:
        return f"{ms / 1000:.2f}s"
    if ms >= 100:
        return f"{ms:.1f}ms"
    return f"{ms:.2f}ms"


def md_row(cells):
    return "| " + " | ".join(str(c) for c in cells) + " |"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tsv", nargs="?", default="bench/data/results.tsv")
    ap.add_argument("--out", default="BENCHMARK.md")
    args = ap.parse_args()
    rows = load(args.tsv)
    cells = {c: {i: rows.get((c, i)) for i in IMPLS} for c, _ in CASES}
    cell = lambda c, i: cells[c].get(i)

    def best(case):
        b = None
        for i in IMPLS:
            v = cell(case, i)
            if v and v[1] == "PASS" and (b is None or v[0] < b[1]):
                b = (i, v[0])
        return b

    out = []
    out.append("# fast_reader Benchmark")
    out.append("")
    out.append("> 数据集：100 万行 JSONL，约 144MB")
    out.append("")

    out.append("## 耗时（中位数，越小越快）")
    out.append("")
    out.append(md_row(["场景"] + IMPLS + ["最快"]))
    out.append(md_row(["---"] * 6))
    for case, desc in CASES:
        r = [case]
        for i in IMPLS:
            v = cell(case, i)
            r.append(("N/A" if v and v[1] == "N/A" else fmt(v[0])) if v else "—")
        b = best(case)
        r.append(fmt(b[1]) + " (" + b[0] + ")" if b else "—")
        out.append(md_row(r))
    out.append("")

    out.append("## 相对耗时（最快者=1.00）")
    out.append("")
    out.append(md_row(["场景"] + IMPLS))
    out.append(md_row(["---"] * 5))
    for case, desc in CASES:
        r = [case]
        b = best(case)
        bm = b[1] if b else 1.0
        for i in IMPLS:
            v = cell(case, i)
            r.append(f"{v[0]/bm:.2f}" if v and v[1] == "PASS" else ("N/A" if v and v[1] == "N/A" else "—"))
        out.append(md_row(r))
    out.append("")

    out.append("## 逐场景最快者")
    out.append("")
    for case, desc in CASES:
        b = best(case)
        if b:
            out.append(f"- **{case}**: {b[0]} ({fmt(b[1])})")
    out.append("")

    # wins
    fr_wins = [c for c, _ in CASES if best(c) and best(c)[0] == "fr"]
    fr_loses = [(c, best(c)[0]) for c, _ in CASES if best(c) and best(c)[0] != "fr"]
    out.append("## 总览")
    out.append(f"- fast_reader 最快: {', '.join(fr_wins) if fr_wins else '无'}")
    out.append(f"- 其余场景胜者: {', '.join(f'{c}({w})' for c, w in fr_loses) if fr_loses else '无'}")
    out.append("")

    with open(args.out, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(out) + "\n")
    print(f"wrote {args.out}")

    for case, desc in CASES:
        b = best(case)
        if b:
            print(f"  {case:<14} -> {b[0]:<4} {fmt(b[1])}")


if __name__ == "__main__":
    main()
