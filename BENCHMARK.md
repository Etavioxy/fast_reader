# fast_reader Benchmark

> 数据集：100 万行 JSONL，约 144MB

## 耗时（中位数，越小越快）

| 场景 | fr | std | cli | py | 最快 |
| --- | --- | --- | --- | --- | --- |
| head | 12.05ms | 9.67ms | 29.09ms | 50.96ms | 9.67ms (std) |
| tail | 9.37ms | 260.8ms | 29.46ms | 246.9ms | 9.37ms (fr) |
| line | 77.00ms | 59.92ms | 261.3ms | 112.1ms | 59.92ms (std) |
| range | 78.46ms | 45.41ms | 252.5ms | 120.7ms | 45.41ms (std) |
| sample | 11.15ms | 245.0ms | 1.87s | 253.1ms | 11.15ms (fr) |
| reverse_line | 42.61ms | 257.6ms | 377.2ms | 253.2ms | 42.61ms (fr) |
| count | 142.9ms | 80.12ms | 92.34ms | 177.8ms | 80.12ms (std) |
| parse | 762.2ms | 720.2ms | N/A | 1.59s | 720.2ms (std) |
| filter | 765.0ms | 708.1ms | 3.35s | 1.61s | 708.1ms (std) |
| aggregate | 761.8ms | 699.8ms | 3.30s | 1.58s | 699.8ms (std) |

## 相对耗时（最快者=1.00）

| 场景 | fr | std | cli | py |
| --- | --- | --- | --- | --- |
| head | 1.25 | 1.00 | 3.01 | 5.27 |
| tail | 1.00 | 27.83 | 3.14 | 26.34 |
| line | 1.29 | 1.00 | 4.36 | 1.87 |
| range | 1.73 | 1.00 | 5.56 | 2.66 |
| sample | 1.00 | 21.96 | 167.32 | 22.69 |
| reverse_line | 1.00 | 6.04 | 8.85 | 5.94 |
| count | 1.78 | 1.00 | 1.15 | 2.22 |
| parse | 1.06 | 1.00 | N/A | 2.20 |
| filter | 1.08 | 1.00 | 4.73 | 2.28 |
| aggregate | 1.09 | 1.00 | 4.71 | 2.25 |

## 逐场景最快者

- **head**: std (9.67ms)
- **tail**: fr (9.37ms)
- **line**: std (59.92ms)
- **range**: std (45.41ms)
- **sample**: fr (11.15ms)
- **reverse_line**: fr (42.61ms)
- **count**: std (80.12ms)
- **parse**: std (720.2ms)
- **filter**: std (708.1ms)
- **aggregate**: std (699.8ms)

## 总览
- fast_reader 最快: tail, sample, reverse_line
- 其余场景胜者: head(std), line(std), range(std), count(std), parse(std), filter(std), aggregate(std)

