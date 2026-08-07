# user-goals

## 性能约束

1. `build_index` / `jump_to_line` / `line_count` 加入后，
   `next_line` / `prev_line` / `current_line` / `random_line` 的 benchmark 中位数性能波动 < 0.1%。
   加入前须先跑 baseline，加入后重跑比对。

2. `build_index` 不得改变 `next_line` / `prev_line` / `current_line` / `random_line` 的内部执行路径。
   索引只赋能 `jump_to_line` 和 `line_count`。

## 架构约束

3. 库零额外必需依赖（仅 std + rand optional）。
