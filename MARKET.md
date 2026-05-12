# 市场调研：timefilter

## 现有竞品分析

| crate | 下载量 | 解析→Duration | 解析→DateTime | 过滤器 | 绝对时间 |
|-------|--------|---------------|---------------|--------|----------|
| **`human-time` 0.1.7** | ~0 | ❌ | ❌ | ❌ | ❌ |
| **`duration-str` 0.21.0** | 7.3M | ✅ | ❌ | ❌ | ❌ |
| **`duration-string` 0.5.3** | 3.5M | ✅ | ❌ | ❌ | ❌ |
| **`timefilter`** (本 crate) | — | ❌ | **✅** | **✅** | **✅** |

## 各竞品不足

### `human-time` 0.1.7
- **只有格式化**（Duration → 人类字符串），没有输入解析
- 基于宏 `elapsed!`，功能极其有限
- 无 DateTime、无过滤器

### `duration-str` 0.21.0
- Duration 解析最强：支持 `y/mon/w/d/h/m/s/ms/µs/ns` 以及 `+` / `*` 表达式
- **只输出 Duration，不输出 DateTime<Utc>**
- **无过滤器算子**
- 依赖 `rust_decimal` 较重

### `duration-string` 0.5.3
- 简单 Duration 解析 + 格式化
- **只输出 Duration**
- **无 DateTime、无过滤器**

## timefilter 差异化优势

1. **相对时间 → DateTime<Utc>（now - duration）** — 独有
2. **绝对时间解析** — `"2024-05-01"`, `"2024-05-01 10:00"`
3. **带运算符的 time filter** — `>=7d`, `<2h` — 独有
4. **TimeOp / TimeFilter 类型** — 独有
5. `format_datetime` UTC → 本地时间
