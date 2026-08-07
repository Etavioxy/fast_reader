# fast_reader Benchmark

> 数据集：100 万行 JSONL，约 144MB

## 耗时（中位数，越小越快）

| 场景 | fr | std | cli | py | 最快 |
| --- | --- | --- | --- | --- | --- |
| head | 0.04ms | 0.03ms | 15.35ms | 33.99ms | 0.03ms (std) |
| tail | 0.06ms | 222.1ms | 14.06ms | 286.2ms | 0.06ms (fr) |
| line | 66.82ms | 47.40ms | 239.3ms | 95.90ms | 47.40ms (std) |
| range | 64.48ms | 34.23ms | 237.2ms | 105.9ms | 34.23ms (std) |
| sample | 1.08ms | 211.9ms | 1.83s | 227.0ms | 1.08ms (fr) |
| reverse_line | 14.77ms | 231.9ms | 352.7ms | 224.8ms | 14.77ms (fr) |
| count | 137.5ms | 78.66ms | 59.07ms | 148.8ms | 59.07ms (cli) |
| parse | 853.8ms | 722.1ms | N/A | 1.56s | 722.1ms (std) |
| filter | 954.8ms | 808.5ms | 3.35s | 1.61s | 808.5ms (std) |
| aggregate | 814.5ms | 883.9ms | 3.28s | 1.54s | 814.5ms (fr) |

## 相对耗时（最快者=1.00）

| 场景 | fr | std | cli | py |
| --- | --- | --- | --- | --- |
| head | 1.24 | 1.00 | 465.00 | 1030.12 |
| tail | 1.00 | 3896.82 | 246.65 | 5021.86 |
| line | 1.41 | 1.00 | 5.05 | 2.02 |
| range | 1.88 | 1.00 | 6.93 | 3.09 |
| sample | 1.00 | 195.26 | 1686.89 | 209.19 |
| reverse_line | 1.00 | 15.70 | 23.88 | 15.22 |
| count | 2.33 | 1.33 | 1.00 | 2.52 |
| parse | 1.18 | 1.00 | N/A | 2.16 |
| filter | 1.18 | 1.00 | 4.14 | 1.99 |
| aggregate | 1.00 | 1.09 | 4.02 | 1.89 |

## 逐场景最快者

- **head**: std (0.03ms)
- **tail**: fr (0.06ms)
- **line**: std (47.40ms)
- **range**: std (34.23ms)
- **sample**: fr (1.08ms)
- **reverse_line**: fr (14.77ms)
- **count**: cli (59.07ms)
- **parse**: std (722.1ms)
- **filter**: std (808.5ms)
- **aggregate**: fr (814.5ms)

## 总览
- fast_reader 最快: tail, sample, reverse_line, aggregate
- 其余场景胜者: head(std), line(std), range(std), count(cli), parse(std), filter(std)

