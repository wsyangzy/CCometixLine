# CCometixLine Segment Options 使用指南

本指南详细介绍 CCometixLine 中所有支持扩展配置选项的 segment 及其使用方法。

## 概述

CCometixLine 支持 6 个主要 segment 的扩展配置选项，每个 segment 都提供丰富的自定义功能：

- [Usage Segment](#usage-segment) - 使用量追踪和进度条显示
- [Git Segment](#git-segment) - Git 状态和分支信息
- [Model Segment](#model-segment) - AI 模型显示和名称映射
- [Session Segment](#session-segment) - 会话时间和持续时间
- [Cost Segment](#cost-segment) - 成本追踪和货币格式
- [Directory Segment](#directory-segment) - 目录路径显示

## 配置语法

所有 segment 都使用统一的 TOML 配置格式：

```toml
[[segments]]
id = "segment_name"
enabled = true

[segments.icon]
plain = "📁"
nerd_font = ""

[segments.colors.icon]
c256 = 215

[segments.colors.text]
c256 = 215

[segments.styles]
text_bold = true

[segments.options]
# segment 特定的配置选项
option_name = "value"
```

---

## Usage Segment

使用量追踪 segment，显示上下文窗口使用情况。

### 基本配置

```toml
[[segments]]
id = "usage"
enabled = true
```

### 扩展选项

| 选项名称                | 类型    | 默认值 | 描述                                                 |
| ----------------------- | ------- | ------ | ---------------------------------------------------- |
| `display_format`      | String  | "both" | 显示格式："percentage"\| "tokens" \| "both" \| "bar" |
| `show_limit`          | Boolean | true   | 是否显示总限制                                       |
| `warning_threshold`   | Number  | 75     | 警告阈值 (0-100)                                     |
| `critical_threshold`  | Number  | 90     | 严重阈值 (0-100)                                     |
| `compact_format`      | Boolean | true   | 紧凑格式显示                                         |
| `token_unit`          | String  | "auto" | 令牌单位："auto"\| "k" \| "raw"                      |
| `bar_show_percentage` | Boolean | true   | 进度条模式下显示百分比                               |
| `bar_show_tokens`     | Boolean | false  | 进度条模式下显示令牌数                               |

### 使用示例

#### 基础百分比显示

```toml
[[segments]]
id = "usage"
enabled = true

[segments.options]
display_format = "percentage"
show_limit = false
compact_format = true
```

显示效果：`85%`

#### 令牌数显示

```toml
[[segments]]
id = "usage"
enabled = true

[segments.options]
display_format = "tokens"
show_limit = true
token_unit = "k"
```

显示效果：`17.2k/20k`

#### 进度条模式

```toml
[[segments]]
id = "usage"
enabled = true

[segments.options]
display_format = "bar"
bar_show_percentage = true
bar_show_tokens = true
show_limit = true
warning_threshold = 80
critical_threshold = 95
```

显示效果：`████████░░ 85% 17.2k/20k`

#### 警告阈值配置

```toml
[[segments]]
id = "usage"
enabled = true

[segments.options]
warning_threshold = 70    # 70% 时显示警告颜色
critical_threshold = 85   # 85% 时显示严重警告颜色
```

---

## Git Segment

Git 状态和分支信息显示。

### 基本配置

```toml
[[segments]]
id = "git"
enabled = true
```

### 扩展选项

| 选项名称              | 类型    | 默认值    | 描述                                    |
| --------------------- | ------- | --------- | --------------------------------------- |
| `show_sha`          | Boolean | false     | 显示提交 SHA                            |
| `sha_length`        | Number  | 7         | SHA 显示长度 (6-12)                     |
| `show_remote`       | Boolean | true      | 显示远程状态                            |
| `show_stash`        | Boolean | true      | 显示 stash 数量                         |
| `show_tag`          | Boolean | true      | 显示最近的 tag                          |
| `hide_clean_status` | Boolean | false     | 隐藏干净状态                            |
| `branch_max_length` | Number  | 20        | 分支名最大长度 (5-50)                   |
| `status_format`     | String  | "symbols" | 状态格式："symbols"\| "text" \| "count" |

### 使用示例

#### 基础分支显示

```toml
[[segments]]
id = "git"
enabled = true

[segments.options]
show_sha = false
show_remote = true
status_format = "symbols"
```

显示效果：`main +2 ~1`

#### 完整信息显示

```toml
[[segments]]
id = "git"
enabled = true

[segments.options]
show_sha = true
sha_length = 8
show_remote = true
show_stash = true
show_tag = true
status_format = "symbols"
branch_max_length = 15
```

显示效果：`main@1a2b3c4d +2 ~1 ↑1 [v1.0.0] {2}`

#### 状态格式对比

```toml
# 符号格式
[segments.options]
status_format = "symbols"
# 显示：main +2 ~1 -1

# 文本格式  
[segments.options]
status_format = "text"
# 显示：main added:2 modified:1 deleted:1

# 计数格式
[segments.options]
status_format = "count"
# 显示：main (4 changes)
```

#### 分支名长度控制

```toml
[[segments]]
id = "git"
enabled = true

[segments.options]
branch_max_length = 10    # 超过10字符的分支名会被截断
```

显示效果：`feature/lo...` (原分支名: feature/long-branch-name)

---

## Model Segment

AI 模型显示和自定义名称映射。

### 基本配置

```toml
[[segments]]
id = "model"
enabled = true
```

### 扩展选项

| 选项名称             | 类型    | 默认值 | 描述                                  |
| -------------------- | ------- | ------ | ------------------------------------- |
| `display_format`   | String  | "name" | 显示格式："name"\| "full" \| "custom" |
| `show_version`     | Boolean | false  | 显示版本信息                          |
| `abbreviate_names` | Boolean | true   | 使用缩写名称                          |
| `custom_names`     | Table   | {}     | 自定义名称映射表                      |

### 使用示例

#### 基础名称显示

```toml
[[segments]]
id = "model"
enabled = true

[segments.options]
display_format = "name"
abbreviate_names = true
```

显示效果：`Sonnet 3.5`

#### 完整名称显示

```toml
[[segments]]
id = "model"
enabled = true

[segments.options]
display_format = "full"
show_version = true
abbreviate_names = false
```

显示效果：`claude-3-5-sonnet-20241022`

#### 自定义名称映射

```toml
[[segments]]
id = "model"
enabled = true

[segments.options]
display_format = "custom"
abbreviate_names = true

# 自定义名称映射
[segments.options.custom_names]
"claude-3-5-sonnet-20241022" = "Claude 3.5"
"claude-4-sonnet-20241022" = "Claude 4"
"gpt-4-0125-preview" = "GPT-4 Turbo"
"gpt-3.5-turbo" = "GPT-3.5"
```

显示效果：`Claude 3.5`

#### 版本信息显示

```toml
[[segments]]
id = "model"
enabled = true

[segments.options]
show_version = true
display_format = "name"
```

显示效果：`Sonnet 3.5 (20241022)`

---

## Session Segment

会话时间和持续时间追踪。

### 基本配置

```toml
[[segments]]
id = "session"
enabled = true
```

### 扩展选项

| 选项名称              | 类型    | 默认值 | 描述                                              |
| --------------------- | ------- | ------ | ------------------------------------------------- |
| `time_format`       | String  | "auto" | 时间格式："auto"\| "short" \| "long" \| "digital" |
| `show_milliseconds` | Boolean | false  | 显示毫秒信息                                      |
| `compact_format`    | Boolean | true   | 紧凑格式显示                                      |
| `show_idle_time`    | Boolean | false  | 显示空闲时间                                      |
| `show_line_changes` | Boolean | true   | 显示代码行变更统计 (+增加数 -删除数)                |

### 使用示例

#### 自动时间格式

```toml
[[segments]]
id = "session"
enabled = true

[segments.options]
time_format = "auto"
compact_format = true
```

显示效果：`5m 23s` (5分23秒)

#### 数字时间格式

```toml
[[segments]]
id = "session"
enabled = true

[segments.options]
time_format = "digital"
show_milliseconds = false
```

显示效果：`05:23` (5分23秒)

#### 隐藏代码行变更统计

```toml
[[segments]]
id = "session"
enabled = true

[segments.options]
time_format = "auto"
show_line_changes = false  # 隐藏 +6697 -345 这样的代码行变更显示
```

显示效果：`4h43m` (只显示时间，不显示 +6697 -345)

#### 长格式显示

```toml
[[segments]]
id = "session"
enabled = true

[segments.options]
time_format = "long"
compact_format = false
```

显示效果：`5 minutes 23 seconds`

#### 毫秒精度显示

```toml
[[segments]]
id = "session"
enabled = true

[segments.options]
time_format = "auto"
show_milliseconds = true
```

显示效果：`5m 23.5s` (5分23.5秒)

#### 空闲时间追踪

```toml
[[segments]]
id = "session"
enabled = true

[segments.options]

```

显示效果：`5m 23s (idle: 1m 15s)`

#### 时间格式对比表

| 格式    | 示例显示                 | 描述                 |
| ------- | ------------------------ | -------------------- |
| auto    | `5m 23s`               | 自动选择最合适的格式 |
| short   | `5m23s`                | 紧凑的短格式         |
| long    | `5 minutes 23 seconds` | 完整的长格式         |
| digital | `05:23`                | 数字时钟格式         |

---

## Cost Segment

成本追踪和货币格式显示。

### 基本配置

```toml
[[segments]]
id = "cost"
enabled = true
```

### 扩展选项

| 选项名称               | 类型    | 默认值 | 描述                                                    |
| ---------------------- | ------- | ------ | ------------------------------------------------------- |
| `currency_format`    | String  | "auto" | 货币格式："auto"\| "fixed" \| "compact" \| "scientific" |
| `precision`          | Number  | 2      | 小数位数 (0-6)                                          |
| `show_breakdown`     | Boolean | false  | 显示成本分解                                            |
| `threshold_warning`  | Number  | 1.0    | 警告阈值                                                |
| `cumulative_display` | Boolean | false  | 累积显示模式                                            |

### 使用示例

#### 自动货币格式

```toml
[[segments]]
id = "cost"
enabled = true

[segments.options]
currency_format = "auto"
precision = 2
```

显示效果：`$0.023`

#### 紧凑格式显示

```toml
[[segments]]
id = "cost"
enabled = true

[segments.options]
currency_format = "compact"
precision = 1
```

显示效果：`2.3¢` (小于1美分时显示为分)

#### 科学计数法格式

```toml
[[segments]]
id = "cost"
enabled = true

[segments.options]
currency_format = "scientific"
precision = 2
```

显示效果：`2.3e-2$`

#### 成本分解显示

```toml
[[segments]]
id = "cost"
enabled = true

[segments.options]
show_breakdown = true
currency_format = "auto"
```

显示效果：`$0.023 (in:$0.015 out:$0.008)`

#### 警告阈值配置

```toml
[[segments]]
id = "cost"
enabled = true

[segments.options]
threshold_warning = 0.5   # 成本达到$0.5时显示警告
currency_format = "auto"
```

#### 货币格式对比表

| 格式       | 示例显示    | 描述                   |
| ---------- | ----------- | ---------------------- |
| auto       | `$0.023`  | 自动选择最合适的格式   |
| fixed      | `$0.023`  | 固定小数位数格式       |
| compact    | `2.3¢`   | 紧凑格式，小额显示为分 |
| scientific | `2.3e-2$` | 科学计数法格式         |

---

## Directory Segment

目录路径显示和格式化。

### 基本配置

```toml
[[segments]]
id = "directory"
enabled = true
```

### 扩展选项

| 选项名称            | 类型    | 默认值     | 描述                                                |
| ------------------- | ------- | ---------- | --------------------------------------------------- |
| `max_length`      | Number  | 20         | 最大显示长度 (5-100)                                |
| `show_full_path`  | Boolean | false      | 显示完整路径                                        |
| `abbreviate_home` | Boolean | true       | 主目录缩写为 ~                                      |
| `show_parent`     | Boolean | false      | 显示父目录                                          |
| `case_style`      | String  | "original" | 大小写样式："original"\| "lowercase" \| "uppercase" |

### 使用示例

#### 基础目录显示

```toml
[[segments]]
id = "directory"
enabled = true

[segments.options]
max_length = 20
abbreviate_home = true
show_full_path = false
```

显示效果：`~/projects/ccline`

#### 完整路径显示

```toml
[[segments]]
id = "directory"
enabled = true

[segments.options]
show_full_path = true
abbreviate_home = false
max_length = 50
```

显示效果：`/Users/username/projects/ccometixline`

#### 父目录显示

```toml
[[segments]]
id = "directory"
enabled = true

[segments.options]
show_parent = true
max_length = 30
```

显示效果：`projects/ccometixline`

#### 大小写样式控制

```toml
[[segments]]
id = "directory"
enabled = true

[segments.options]
case_style = "uppercase"
max_length = 15
```

显示效果：`CCOMETIXLINE`

#### 路径长度截断

```toml
[[segments]]
id = "directory"
enabled = true

[segments.options]
max_length = 10
show_full_path = true
```

显示效果：`...ixline` (原路径过长时自动截断)

#### 大小写样式对比表

| 样式      | 示例显示         | 描述           |
| --------- | ---------------- | -------------- |
| original  | `CCometixLine` | 保持原始大小写 |
| lowercase | `ccometixline` | 全部小写       |
| uppercase | `CCOMETIXLINE` | 全部大写       |

---

## 最佳实践

### 1. 性能优化建议

- **Git Segment**: 在大型仓库中，考虑关闭 `show_stash` 和 `show_tag` 以提升性能
- **Usage Segment**: 使用 `compact_format = true` 减少显示宽度
- **Directory Segment**: 设置合理的 `max_length` 避免路径过长

### 2. 显示效果优化

- **进度条模式**: 建议同时启用 `bar_show_percentage` 和 `bar_show_tokens` 获得最佳信息密度
- **时间格式**: 对于短时间会话使用 `digital` 格式，长时间会话使用 `auto` 格式
- **成本显示**: 小额成本使用 `compact` 格式，大额成本使用 `auto` 格式

### 3. 主题一致性

- 确保所有 segment 的颜色配置保持一致
- 使用相同的图标风格（plain 或 nerd_font）
- 保持分隔符样式统一

### 4. 配置模板

#### 最小化配置

```toml
[[segments]]
id = "usage"
enabled = true
[segments.options]
display_format = "percentage"
compact_format = true

[[segments]]
id = "git"
enabled = true
[segments.options]
status_format = "symbols"
show_sha = false

[[segments]]
id = "directory"
enabled = true
[segments.options]
max_length = 15
abbreviate_home = true
```

#### 完整功能配置

```toml
# 详细使用量显示
[[segments]]
id = "usage"
enabled = true
[segments.options]
display_format = "bar"
bar_show_percentage = true
bar_show_tokens = true
show_limit = true
warning_threshold = 75
critical_threshold = 90

# 丰富Git信息
[[segments]]
id = "git"
enabled = true
[segments.options]
show_sha = true
sha_length = 7
show_remote = true
show_stash = true
show_tag = true
status_format = "symbols"

# 自定义模型名称
[[segments]]
id = "model"
enabled = true
[segments.options]
display_format = "custom"
abbreviate_names = true
[segments.options.custom_names]
"claude-3-5-sonnet-20241022" = "Claude 3.5"

# 详细会话信息
[[segments]]
id = "session"
enabled = true
[segments.options]
time_format = "auto"
show_idle_time = true
compact_format = true

# 成本追踪
[[segments]]
id = "cost"
enabled = true
[segments.options]
currency_format = "auto"
show_breakdown = true
threshold_warning = 1.0

# 目录显示
[[segments]]
id = "directory"
enabled = true
[segments.options]
max_length = 25
show_parent = true
case_style = "original"
```

## 7. OutputStyle 段

### 功能描述

显示当前 Claude Code 的输出样式配置，支持多种显示格式和自定义名称映射。

### 配置选项

| 选项名称             | 类型    | 默认值 | 描述           | 可选值                                  |
| -------------------- | ------- | ------ | -------------- | --------------------------------------- |
| `display_format`   | String  | "name" | 显示格式       | "name", "full", "abbreviated", "custom" |
| `abbreviate_names` | Boolean | false  | 使用缩写名称   | true, false                             |
| `show_description` | Boolean | false  | 显示样式描述   | true, false                             |
| `custom_names`     | Object  | {}     | 自定义名称映射 | 键值对对象                              |

### 配置示例

```toml
[[segments]]
id = "output_style"
enabled = true

[segments.icon]
plain = "🎯"
nerd_font = ""

[segments.colors]
icon = { c16 = 6 }  # Cyan
text = { c16 = 6 }

[segments.options]
display_format = "abbreviated"
abbreviate_names = true
show_description = false
# custom_names = { "engineer-professional" = "Eng-Pro" }
```

### 显示格式说明

- **name**: 显示原始样式名称
  - 示例：`engineer-professional`
- **full**: 显示完整信息，可选择性包含描述
  - 示例：`engineer-professional` 或 `engineer-professional (output style)`
- **abbreviated**: 显示缩写名称
  - 示例：`Eng-Pro`
- **custom**: 使用自定义名称映射
  - 示例：根据 `custom_names` 配置显示

### 内置样式缩写映射

| 原始名称              | 缩写     |
| --------------------- | -------- |
| engineer-professional | Eng-Pro  |
| creative              | Creative |
| concise               | Concise  |
| detailed              | Detail   |
| technical             | Tech     |
| casual                | Casual   |
| formal                | Formal   |
| academic              | Academic |
| tutorial              | Tutorial |

### 样式描述支持

当 `show_description = true` 且 `display_format = "full"` 时，会在次要显示区域显示样式描述：

| 样式名称              | 描述                              |
| --------------------- | --------------------------------- |
| engineer-professional | Professional engineering style    |
| creative              | Creative and expressive style     |
| concise               | Brief and to-the-point style      |
| detailed              | Comprehensive and thorough style  |
| technical             | Technical documentation style     |
| casual                | Informal and conversational style |
| formal                | Formal business style             |
| academic              | Academic writing style            |
| tutorial              | Step-by-step tutorial style       |

### 自定义名称映射

使用 `custom_names` 选项可以为任何样式定义自定义显示名称：

```toml
[segments.options]
display_format = "custom"

# TOML 格式的自定义名称映射
[segments.options.custom_names]
"engineer-professional" = "🛠️ Eng"
"creative" = "🎨 Art"
"concise" = "⚡ Fast"
"detailed" = "📋 Full"
```

---

## 故障排除

### 常见问题

1. **配置不生效**: 检查 TOML 语法是否正确，特别注意引号和数据类型
2. **显示异常**: 确认终端支持 Nerd Font 图标
3. **性能问题**: 在大型 Git 仓库中关闭耗时的选项

### 配置验证

使用以下命令验证配置文件：

```bash
ccline --check    # 检查配置有效性
ccline --print    # 打印当前配置
```

### 重置配置

如果配置出现问题，可以重置到默认值：

```bash
ccline --init     # 重新初始化配置文件
```

---

## 参考资源

- [完整配置示例](example_enhanced.toml)
- [核心模块文档](src/core/CLAUDE.md)
- [UI 模块文档](src/ui/CLAUDE.md)
- [主项目文档](README.md)
