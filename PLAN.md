# timefilter crate 计划

## 来源

从 [fsmon](../fsmon/) 的 `src/utils.rs` 中提取的时间解析/过滤功能。

## 外部依赖

- `chrono` — `DateTime<Utc>` 时间处理

## 步骤

### 1. 目录结构

```
timefilter/
├── Cargo.toml
├── src/
│   ├── lib.rs  — crate 根 + prelude
│   └── time.rs — 全部逻辑
```

### 2. 导出 API

```rust
pub use time::{TimeOp, TimeFilter, TimeError, TimeResult};
pub use time::{parse_time, parse_time_filter, format_datetime};
```

- `TimeOp` — `Gt` / `Ge` / `Lt` / `Le` / `Eq`（独立于 `SizeOp`）
- `TimeFilter` — `{ op: TimeOp, time: DateTime<Utc> }`
- `parse_time(s)` — 相对/绝对时间 → `DateTime<Utc>`
- `parse_time_filter(s)` — `">=7d"`, `"<2h"` → `TimeFilter`
- `format_datetime(dt)` — UTC → 本地时间字符串

### 3. 设计决策

- `TimeOp` 与 `SizeOp` 独立（PROGRESS.md 推荐方案 A）
- 相对时间用 `eq_ignore_ascii_case`（零分配），不用 `to_lowercase()`
- 所有错误字符串在 `.rodata`

### 4. 测试

25 unit + 5 doc = 30 tests
