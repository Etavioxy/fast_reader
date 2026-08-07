# user-goals

## 性能约束

1. `build_index` 调用后，`next_line` / `prev_line` / `current_line` / `random_line`
   的性能**不得劣化**（允许不变或变快，禁止变慢）。
   benchmark 重场景（count/parse/filter/aggregate）中位数不得上升。

2. `build_index` 是一次性 O(N) 扫描。索引用于需要随机跳转的操作：
   `jump_to_line`（O(1)）、`random_line`（均匀分布）、`line_count`。
   `next_line` / `prev_line` 保持缓冲/窗口路径（实测索引快路径慢 5-20%，
   因逐行 seek 破坏顺序访问局部性）。

3. `jump_to_line` / `line_count` 仅在 `build_index` 后可用。

## 架构约束

4. 库零额外必需依赖（仅 std + rand optional）。
