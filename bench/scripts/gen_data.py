#!/usr/bin/env python3
"""Generate a realistic JSONL dataset for the benchmark.

Fields: id, user{name,age}, score, active, city, tags, ts.
Deterministic via --seed. Size ~ 170 bytes/line at 1M lines.
"""
import argparse
import json
import random


def gen(out: str, lines: int, seed: int) -> None:
    rng = random.Random(seed)
    cities = ["Shanghai", "Beijing", "Shenzhen", "Guangzhou",
              "Hangzhou", "Chengdu", "Wuhan", "Xian"]
    tags_pool = ["alpha", "beta", "gamma", "delta", "epsilon",
                 "zeta", "eta", "theta"]
    with open(out, "w", encoding="utf-8", newline="\n") as f:
        for i in range(lines):
            rec = {
                "id": i,
                "user": {
                    "name": f"user_{i:07d}",
                    "age": rng.randint(18, 70),
                },
                "score": round(rng.uniform(0, 1000), 2),
                "active": rng.random() < 0.5,
                "city": rng.choice(cities),
                "tags": rng.sample(tags_pool, rng.randint(1, 4)),
                "ts": 1620000000 + i,
            }
            f.write(json.dumps(rec, separators=(",", ":")) + "\n")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="data/sample.jsonl")
    ap.add_argument("--lines", type=int, default=1_000_000)
    ap.add_argument("--seed", type=int, default=42)
    args = ap.parse_args()
    gen(args.out, args.lines, args.seed)
    print(f"generated {args.lines} lines -> {args.out}")
