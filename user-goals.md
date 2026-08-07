# user-goals

## 性能约束

1. `build_index` 调用后，`next_line` / `prev_line` / `current_line` / `random_line`
   的性能**不得劣化**（允许不变或变快，禁止变慢）。
   benchmark 重场景（count/parse/filter/aggregate）中位数不得上升。

2. `build_index` 是增量扫描（O(已读→EOF)），补齐 `next_line` 未覆盖的剩余行。
   索引用于需要随机跳转的操作：`jump_to_line`（O(1)）、`random_line`（均匀分布）、`line_count`。
   `next_line` / `prev_line` 保持缓冲/窗口路径（实测索引快路径慢 5-20%，
   因逐行 seek 破坏顺序访问局部性）。

3. 索引用永久增量维护（`next_line` 无条件推入偏移，无 flag 切换）。
   `jump_to_line(n)` 对 n ≤ index.len() 总是可用；`line_count` 返当前已知行数。
   `build_index` 补齐未读行（增量扫描从 index 末尾到 EOF），非全量重建。
   去重：`next_line` 仅在 frontier（`current_line_num == index.len()`）推入，
   避免 bof→next_line 重读已索引行时的重复条目。

## 架构约束

4. 库零额外必需依赖（仅 std + rand optional）。
