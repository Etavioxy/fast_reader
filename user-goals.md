# user-goals

## 性能约束

1. `build_index` 调用后，`next_line` / `prev_line` / `current_line` / `random_line`
   的性能**不得劣化**（允许不变或变快，禁止变慢）。
   benchmark 重场景（count/parse/filter/aggregate）中位数不得上升。

2. `build_index` 是一次性 O(N) 扫描。扫描完成后，索引应被所有后续操作复用，
   不得闲置。包括但不限于：next_line 可用索引代替缓冲扫描，prev_line 可用
   索引代替窗口倒读，random_line 可用索引做均匀分布。

3. `jump_to_line` / `line_count` 仅在 `build_index` 后可用。

## 架构约束

4. 库零额外必需依赖（仅 std + rand optional）。
