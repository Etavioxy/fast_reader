#!/usr/bin/env python3
"""Python reference implementation for the easy_reader JSONL benchmark.

Usage:
    python bench/py_runner.py <case> <data> [params...]

Cases:
    head  N          print first N lines
    tail  N          print last N lines
    line  N          print line N (1-based)
    range A B        print lines A..B (1-based, inclusive)
    sample K         print K randomly sampled lines
    count            print number of lines
    parse            print count of lines that parse as JSON
    filter           print count of lines with score > 500
    aggregate        print sum of score (2 decimals)
"""
import json
import random
import sys


def main() -> None:
    case = sys.argv[1]
    data = sys.argv[2]
    params = sys.argv[3:]
    p = lambda i: int(params[i])

    if case == "head":
        n = p(0)
        with open(data, encoding="utf-8") as f:
            for i, line in enumerate(f):
                if i >= n:
                    break
                sys.stdout.write(line)
    elif case == "tail":
        n = p(0)
        with open(data, encoding="utf-8") as f:
            lines = f.readlines()
        sys.stdout.writelines(lines[-n:])
    elif case == "line":
        n = p(0)
        with open(data, encoding="utf-8") as f:
            for i, line in enumerate(f):
                if i == n - 1:
                    sys.stdout.write(line)
                    break
    elif case == "range":
        a, b = p(0), p(1)
        with open(data, encoding="utf-8") as f:
            for i, line in enumerate(f):
                if a - 1 <= i <= b - 1:
                    sys.stdout.write(line)
                elif i > b - 1:
                    break
    elif case == "sample":
        k = p(0)
        with open(data, encoding="utf-8") as f:
            lines = f.readlines()
        for line in random.Random(1).sample(lines, k):
            sys.stdout.write(line)
    elif case == "reverse_line":
        n = p(0)
        with open(data, encoding="utf-8") as f:
            lines = f.readlines()
        sys.stdout.write(lines[len(lines) - n])
    elif case == "count":
        n = 0
        with open(data, encoding="utf-8") as f:
            for _ in f:
                n += 1
        print(n)
    elif case == "parse":
        ok = 0
        with open(data, encoding="utf-8") as f:
            for line in f:
                json.loads(line)
                ok += 1
        print(ok)
    elif case == "filter":
        c = 0
        with open(data, encoding="utf-8") as f:
            for line in f:
                if json.loads(line)["score"] > 500:
                    c += 1
        print(c)
    elif case == "aggregate":
        s = 0.0
        with open(data, encoding="utf-8") as f:
            for line in f:
                s += json.loads(line)["score"]
        print(f"{s:.2f}")
    else:
        sys.stderr.write(f"unknown case: {case}\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
