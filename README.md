# Rust File Explorer (rfe)

一个高性能、跨平台的命令行文件浏览器，使用 Rust 编写，提供直观的彩色界面、文件类型图标和丰富的文件操作功能，让你在终端中也能高效管理文件。

## ✨ 核心功能

rfe 围绕 **高效**、**好用**、**好玩** 三个维度，提供完整的终端文件管理能力。

### 🧰 基础文件操作

- **目录浏览**：彩色输出与文件类型 emoji 图标，支持隐藏文件、详细信息（大小 / 创建时间 / 修改时间）的展示
- **文件操作**：移动、复制（`mv --cp`）、使用系统默认应用打开（`open`）
- **创建工具**：`mkdf` 命令一站式创建文件 / 文件夹，自动补齐父目录
- **剪贴板集成**：一键复制当前目录或文件绝对路径
- **正则搜索**：`ls --re` 全局正则搜索，支持递归与大小写不敏感匹配

### ⚙️ 高效语法

- **路径别名（`@`）**：自定义短别名替代长路径，全局可用、持久化、可补全
- **命令链式执行（`->`** **/** **`->!`）**：多命令串联，前序输出自动注入后续命令，支持容错节点
- **占位符扩展（`{}`）**：将前序输出插入到任意参数位置，支持多次引用
- **路径层级弹出（`{}.pop...`）**：在占位符后追加 `.pop`，每多一个 `.pop` 向上回退一级目录，自动处理边界
- **标签管理**：为文件添加多标签，支持正则查询、批量筛选、自动备份
- **标签目录跳转（`cd -idx`）**：基于 `.index` 文件约定，通过标签快速跳转到目标目录
- **目录历史返回（`cd -b`** **/** **`cd -back`）**：一键返回上一个工作目录

### 🎨 体验增强

- **双运行模式**：交互式 REPL（含命令补全、历史记录、ESC 一键清空）与直接命令执行
- **Moe Moe 模式（`-moe`）**：粉色系配色 + 智能颜文字，可通过 `change` 动态切换
- **Welcome 命令**：随时重新显示欢迎页面，自动适配当前模式
- **跨平台兼容**：完美支持 Windows / Linux / macOS
- **轻量高性能**：Rust 原生编译，启动迅速、资源占用低

## 🔑 核心概念与进阶语法

rfe 在标准 CLI 命令之上引入了一套**可组合的工作流语法**。理解以下四个核心概念，将显著提升日常文件操作效率：

| 语法             | 名称   | 一句话说明            |
| -------------- | ---- | ---------------- |
| `@<alias>`     | 路径别名 | 用短名称代替任意长路径      |
| `cmd1 -> cmd2` | 命令链  | 串联多个命令，前序输出自动传递  |
| `{}`           | 占位符  | 把前序输出插入到任意参数位置   |
| `{}.pop...`    | 路径弹出 | 在前序路径上向上回退 N 级目录 |

它们可以自由组合，例如 `cppwd -> alias add proj {}.pop` 表示「取当前路径的父目录，添加为名为 `proj` 的别名」。

***

### 🔖 `@` — 路径别名（Path Alias）

为常用目录设置短别名，之后用 `@别名` 即可瞬间定位，无需输入冗长路径。

```bash
# 管理别名
rfe alias add <name> <path>   # 添加 / 更新
rfe alias remove <name>       # 删除
rfe alias list                # 列出全部

# 使用别名（在任何接收路径的命令中均可）
rfe cd @proj
rfe ls @proj/src/components
rfe open @dl/report.pdf
```

**关键特性**

- 全局可用：兼容 `ls`、`cd`、`open`、`cpf`、`mv` 等所有路径相关命令
- 子路径拼接：支持 `@别名/子路径` 形式深入目录
- 持久化存储：自动保存至系统配置目录，重启后依旧有效
  - Windows：`%APPDATA%\rfe\aliases.json`
  - Linux / macOS：`~/.config/rfe/aliases.json`
- 交互式补全：REPL 中输入 `@` + Tab 可自动补全已有别名

***

### 🔗 `->` — 命令链式执行（Command Chain）

通过 `->` 将多个命令串联，前一个命令的**原始输出**会作为输入数据传递给下一个命令，实现复杂多步操作的"一行式"表达。

```bash
cmd1 -> cmd2 -> cmd3   # 顺序执行；任一命令失败则中断
cmd1 ->! cmd2          # 容错节点：cmd1 失败也继续执行 cmd2
```

**典型示例**

```bash
# 1. 浏览当前目录后跳到上级目录
rfe pwd -> ls -> cd .. -> pwd

# 2. 正则搜索 README 并复制其绝对路径到剪贴板
rfe ls --re "^README\.md$" -> cpf

# 3. 搜索并打开 .rs 文件
rfe ls --re "\.rs$" -> open

# 4. 容错示例：目录不存在也继续 ls
rfe cd maybe_nonexist ->! ls
```

**关键特性**

- 顺序执行 + 数据管道：前序原始输出自动作为下一命令的首参数
- 双重错误策略：`->` 严格中断 / `->!` 容错继续
- 兼容所有 rfe 命令，REPL 与直接模式均可用

***

### 📍 `{}` — 占位符扩展（Placeholder Expansion）

默认情况下前序输出作为下一命令的首参数。使用 `{}` 可将其插入到**任意位置**或**多次引用**。

```bash
cmd1 -> cmd2 <arg1> {} <arg3>   # 插入到中间位置
cmd1 -> cmd2 {} {}              # 多次引用同一输出
```

**典型示例**

```bash
# 用当前路径快速添加别名（输出位于命令尾部）
rfe cppwd -> alias add desktop {}

# 复制文件路径，并为该文件添加多个标签
rfe cpf main.rs -> tag add {} rust code
```

**关键特性**

- 精准定位：可在命令中任意位置插入
- 多重引用：同一输出可在多处被替换
- 默认传递保留：未写 `{}` 时仍按默认规则注入到首参数

***

### 📂 `{}.pop` — 路径层级弹出（Path Level Pop）

在占位符后追加任意数量的 `.pop`，表示在路径上**向上回退对应层级**。每多一个 `.pop` 即向上一级。

```bash
cmd -> cd {}.pop           # 上一级（父目录）
cmd -> cd {}.pop.pop       # 上两级
cmd -> cd {}.pop.pop.pop   # 上三级，以此类推
```

**典型示例**

```bash
# 从某个深层文件快速跳到项目根
rfe cpf src/utils/mod.rs -> cd {}.pop.pop.pop

# 取当前目录的祖父目录并设为别名
rfe pwd -> alias add ancestor {}.pop.pop
```

**关键特性**

- 任意级联：理论上无层级上限
- 边界安全：超出实际层级时自动停在最顶层并友好提示，不报错
- 跨平台：兼容 Windows 与 Unix 路径
- 通用：可与 `cd`、`ls`、`open`、`mv` 等任意路径命令配合

***

### 📦 `mkdf` — 文件 / 文件夹创建命令

一站式创建工具，自动处理父目录。

```bash
mkdf -f <path>       # 创建文件，自动补齐父目录
mkdf -d <path>       # 创建文件夹
mkdf -d -p <path>    # 创建多级嵌套文件夹
mkdf -h / --help     # 查看帮助
```

**关键特性**

- 文件 / 文件夹二合一：`-f` 创建文件、`-d` 创建文件夹
- 自动父目录：创建文件时自动补齐缺失的父级目录
- 路径灵活：兼容绝对路径、相对路径与别名路径

***

### 🔙 `cd -b` / `cd -back` — 快速返回上一个目录

无需记忆路径即可在两个工作目录间快速来回切换。

```bash
cd /path/to/dir1
cd /path/to/dir2
cd -b      # 返回 dir1
cd -back   # 再次返回 dir2
```

**关键特性**

- 短 / 长两种形式：`-b` 与 `-back` 等价
- 自动记录历史，错误时友好提示
- 完全兼容现有 `cd`、`cd ..`、`cd ~` 及命令链场景

***

### 🔖 `cd -idx <tag>` — 基于标签的目录跳转

借助标签系统的 `.index` 文件约定，将"目录"纳入标签体系，实现按标签跳转。

```bash
cd -idx <tag>         # 跳转到指定标签的目录
cd -idx "rust|proj"   # 支持正则匹配
```

当多个目录匹配时，会显示交互式选择列表：

```text
🔍 Multiple directories found:
  1. /projects/rust-file-explorer -> rust work important
  2. /projects/game-project -> game fun rust
📍 Enter selection number: _
```

**前置约定**：在目标目录下创建 `.index` 文件并为其打标签即可。详见下方 [💡 使用技巧](#-使用技巧)。

**关键特性**

- 完整正则语法支持
- 多结果交互选择
- 跳转后自动更新历史，支持 `cd -b` 回退

***

### ⌨️ ESC 键 — REPL 输入快速清空

在 REPL 模式下，按 ESC 键可瞬间清空当前输入行内容，无需逐字符退格。常用于：误输入、长输入重来、临时放弃命令。

***

### 🌸 `-moe` — Moe Moe 萌系模式

启用后所有输出统一切换为粉色系配色（RGB: 255, 105, 180）并附加场景化颜文字（💖、🌸、✨、😢、👋 等），让终端操作更治愈～

```bash
rfe -moe              # 以萌系模式进入 REPL
rfe -moe <command>    # 以萌系模式执行单条命令
change -moe           # REPL 中动态切换到萌系模式
change -std           # 切回标准模式
```

**关键特性**

- 全命令覆盖，纯视觉增强，不影响任何原有功能
- 欢迎语：`ciallo∠・ω⌒☆ Welcome to the moe moe mode！`
- 运行时可动态切换，无需重启

***

## 🛠️ 技术栈与环境要求

### 运行 / 编译环境

| 项目    | 要求                                                 |
| ----- | -------------------------------------------------- |
| Rust  | 1.65.0 及以上                                         |
| Cargo | 与 Rust 同步发布                                        |
| 操作系统  | Windows 10+ / Linux（内核 4.15+） / macOS 11+（Big Sur） |

### 核心依赖（节选自 [Cargo.toml](file:///c:/Users/q/Desktop/rust-file-explorer/Cargo.toml)）

| 依赖                     | 版本   | 用途                  |
| ---------------------- | ---- | ------------------- |
| `colored`              | 2.1  | 终端彩色输出              |
| `crossterm`            | 0.28 | 跨平台终端按键事件（ESC 清空等）  |
| `rustyline`            | 12.0 | REPL 行编辑、补全与历史记录    |
| `arboard`              | 3.4  | 跨平台剪贴板访问            |
| `regex`                | 1.10 | 正则搜索与标签匹配           |
| `open`                 | 5.0  | 调用系统默认程序打开文件        |
| `dirs`                 | 5.0  | 获取跨平台配置 / 主目录路径     |
| `unicode-width`        | 0.1  | Unicode 字符宽度计算，优化对齐 |
| `serde` / `serde_json` | 1.0  | 别名、标签数据的 JSON 持久化   |
| `tempfile`             | 3.10 | 标签备份等场景的临时文件管理      |

## 📦 安装步骤

### 第一步：获取并编译源码

克隆仓库、进入项目目录并编译发布版本，编译产物位于 `target/release/rfe`（Windows 下为 `rfe.exe`）：

```bash
git clone https://github.com/glacier-bot/rust-file-explorer.git
cd rust-file-explorer
cargo build --release
```

### 第二步：将 rfe 接入系统 PATH

根据你的操作系统，选择对应的部署方式，使 `rfe` 命令可在终端任意位置直接调用。

#### 🐧 Linux / macOS

将编译产物复制到系统二进制目录即可：

```bash
sudo cp target/release/rfe /usr/local/bin/
```

#### 💻 Windows（PowerShell）

以下三种方式任选其一，**推荐方式 B**，无需管理员权限且配置灵活。

- **方式 A：复制到系统目录（最简单，需管理员权限）**

  将 `target\release\rfe.exe` 复制到 `C:\Windows\System32` 或任意已加入 `PATH` 的目录。
- **方式 B：将编译目录加入用户 PATH（推荐，永久生效）**

  在 PowerShell 中执行（请替换为你的实际路径）：

```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\path\to\rust-file-explorer\target\release", [EnvironmentVariableTarget]::User)
```

执行后需关闭并重新打开 PowerShell 窗口使其生效。

- **方式 C：在 PowerShell 配置文件中设置别名**

  适合不希望修改 PATH 的场景，通过 `$PROFILE` 定义别名：

```powershell
if (!(Test-Path -Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE -Force }
Add-Content $PROFILE 'Set-Alias -Name rfe -Value "C:\path\to\rust-file-explorer\target\release\rfe.exe"'
. $PROFILE
```

> 💡 **常见问题**：若 PowerShell 提示"无法加载文件，因为在此系统上禁止运行脚本"，请以管理员身份运行 `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser` 解除限制。

### 第三步：验证安装

在任意终端（Bash / PowerShell / Zsh 等）中执行以下命令，能正常显示帮助信息即表示安装成功：

```bash
rfe help
```

## 🚀 使用指南

### 运行模式

| 模式           | 启动方式                   | 适用场景                         |
| ------------ | ---------------------- | ---------------------------- |
| **交互式 REPL** | `rfe`                  | 连续多次操作；享受补全、历史记录、ESC 清空等交互特性 |
| **单次命令**     | `rfe <command> [args]` | 脚本调用、单次操作、与其他 CLI 工具组合使用     |

> 💡 所有命令在两种模式下行为一致；REPL 中无需 `rfe` 前缀。

### 命令速查表

下表按命令族分组，覆盖 rfe 当前全部命令。`ls`、`cd` 等命令的子选项采用「主命令 + 选项」的形式列出，便于查阅。

#### 📁 目录浏览（`ls`）

| 用法                                       | 说明                                            |
| ---------------------------------------- | --------------------------------------------- |
| `ls`                                     | 列出当前目录内容                                      |
| `ls <path>`                              | 列出指定目录内容                                      |
| `ls -a`                                  | 同时显示隐藏文件                                      |
| `ls -l`                                  | 显示详细信息（大小、创建 / 修改时间）                          |
| `ls -la`                                 | 等同 `-a -l`                                    |
| `ls --re <pattern>`                      | 正则全局搜索（默认从当前目录）                               |
| `ls --re-deep <pattern>`                 | 递归正则搜索                                        |
| `ls --re --xcaps <pattern>`              | 大小写不敏感正则搜索（`--xcaps` 与 `--re-insensitive` 等价） |
| `ls --re-deep --xcaps <pattern>`         | 递归 + 大小写不敏感                                   |
| `ls -tag` / `ls --tags`                  | 列表中附带显示每个文件的标签                                |
| `ls -t <pattern>` / `ls --tag <pattern>` | 按标签过滤当前目录文件，可重复传参组合多标签                        |
| `ls -t --deep <pattern>`                 | 递归按标签过滤，向下遍历所有子目录                             |

#### 📍 路径与导航（`pwd` / `cd`）

| 用法                   | 说明                             |
| -------------------- | ------------------------------ |
| `pwd`                | 打印当前工作目录                       |
| `cd` / `cd ~`        | 切换到用户主目录                       |
| `cd ..`              | 切换到上级目录                        |
| `cd <path>`          | 切换到指定目录                        |
| `cd -b` / `cd -back` | 返回上一个工作目录                      |
| `cd -idx <tag>`      | 通过 `.index` 文件标签跳转目录，支持正则与交互选择 |

#### 📋 剪贴板（`cppwd` / `cpf`）

| 用法           | 说明                |
| ------------ | ----------------- |
| `cppwd`      | 复制当前目录绝对路径到系统剪贴板  |
| `cpf <file>` | 复制指定文件的绝对路径到系统剪贴板 |

#### 🗂️ 文件操作（`mv` / `open` / `mkdf`）

| 用法                     | 说明                       |
| ---------------------- | ------------------------ |
| `mv <src> <dest>`      | 移动文件 / 目录                |
| `mv <src> <dest> --cp` | 复制文件 / 目录（保留原文件）         |
| `open <path>`          | 用系统默认应用打开文件，或在资源管理器中打开目录 |
| `mkdf -f <path>`       | 创建文件，自动补齐父目录             |
| `mkdf -d <path>`       | 创建文件夹                    |
| `mkdf -d -p <path>`    | 创建多级嵌套文件夹                |
| `mkdf -h` / `--help`   | 查看 `mkdf` 帮助             |

#### 🔖 路径别名（`alias` / `@`）

| 用法                        | 说明           |
| ------------------------- | ------------ |
| `alias add <name> <path>` | 添加 / 更新别名    |
| `alias remove <name>`     | 删除别名         |
| `alias list`              | 查看全部别名       |
| `@<name>[/subpath]`       | 在任意路径参数中引用别名 |

#### 🏷️ 标签管理（`tag` / `t`）

| 用法                                   | 说明                   |
| ------------------------------------ | -------------------- |
| `tag add <file> <tag1> [tag2...]`    | 为文件添加一个或多个标签         |
| `tag remove <file> <tag1> [tag2...]` | 删除指定标签               |
| `tag clear <file>`                   | 清空该文件的全部标签           |
| `tag get <file>`                     | 查看文件的标签              |
| `tag list`                           | 列出所有带标签的文件           |
| `tag find <pattern1> [pattern2...]`  | 全局按标签搜索文件（正则、可多条件组合） |
| `tag backup` / `tag restore`         | 手动备份 / 恢复标签数据        |

> 💡 `tag` 命令也支持简写形式 `t`，例如 `t add main.rs rust`。`alias`、`tag` 的部分子命令额外接受常见别名：`alias add` ≡ `alias set`、`alias remove` ≡ `alias rm` ≡ `alias delete`、`alias list` ≡ `alias ls`、`tag remove` ≡ `tag rm`、`tag list` ≡ `tag ls`、`tag find` ≡ `tag search`。

#### 🔗 命令链与占位符

| 用法                    | 说明               |
| --------------------- | ---------------- |
| `cmd1 -> cmd2 -> ...` | 链式执行；前序输出注入下一命令  |
| `cmd1 ->! cmd2`       | 容错节点：前序失败也继续执行   |
| `{}`                  | 在下一命令的任意位置插入前序输出 |
| `{}.pop[.pop...]`     | 在前序路径上向上回退 N 级目录 |

#### 🎨 模式与界面

| 用法                             | 说明                 |
| ------------------------------ | ------------------ |
| `welcome`                      | 重新显示欢迎页面（自动适配当前模式） |
| `clear` / `cls`                | 清空终端屏幕             |
| `help` / `?`                   | 显示帮助信息             |
| `-moe` / `--moe`               | 启用萌系模式（启动参数）       |
| `change -std` / `change --std` | REPL 中切换为标准模式      |
| `change -moe` / `change --moe` | REPL 中切换为萌系模式      |
| `ESC`                          | REPL 模式下清空当前输入行    |
| `exit` / `quit` / `q`          | 退出 REPL            |

### 📝 正则表达式速查

`ls --re` / `tag find` / `cd -idx` 均使用 Rust [`regex`](https://docs.rs/regex) 语法。常用元字符如下：

| 语法                 | 说明                | 示例                         |
| ------------------ | ----------------- | -------------------------- |
| `.`                | 匹配任意单个字符          | `fi.e` → file / fine       |
| `*` / `+` / `?`    | 0+ / 1+ / 0 或 1 次 | `colou?r` → color / colour |
| `^` / `$`          | 字符串起始 / 结尾        | `\.rs$` → 所有 `.rs` 文件      |
| `[abc]` / `[^abc]` | 字符集（取反）           | `[Ff]ile` → File / file    |
| `\|`               | 或逻辑               | `\.rs$\|\.toml$`           |
| `()`               | 分组                | `(src\|target)/`           |

> 💡 `ls --re` 默认仅搜索当前目录的直接条目，使用 `--re-deep` 可递归；匹配结果显示为相对当前目录的路径。

### 💾 配置文件位置

别名与标签数据均以 JSON 形式持久化到系统配置目录，重启后保留。每次标签修改会自动生成 `.bak` 备份，防止误操作丢失数据。

| 平台            | 路径                                                               |
| ------------- | ---------------------------------------------------------------- |
| Windows       | `%APPDATA%\rfe\aliases.json`、`%APPDATA%\rfe\tags.json`（含 `.bak`） |
| Linux / macOS | `~/.config/rfe/aliases.json`、`~/.config/rfe/tags.json`（含 `.bak`） |

***

## 💡 使用技巧

### 🏷️ 为文件夹打标签：`.index` 文件约定

rfe 的标签系统作用于**文件**而非目录。借助一个简单的约定，可让目录也参与标签体系，并配合 `cd -idx` 实现按标签跳转。

**约定**：在需要打标签的目录下创建一个 `.index` 文件，并为该文件添加标签。`tag find`、`ls -t`、`cd -idx` 均能识别此约定。

```bash
# 1. 在目标目录下创建 .index 占位文件
mkdf -f /path/to/folder/.index

# 2. 为 .index 文件打标签
tag add /path/to/folder/.index work project important

# 3. 按标签跳转 / 检索
cd -idx work               # 直接跳转
tag find work project      # 全局搜索匹配的目录
ls -t work                 # 当前目录下按标签过滤
```

**优势**

- 语义清晰：`.index` 即"目录索引文件"
- 跨平台：以 `.` 开头的隐藏文件在三大系统通用
- 易管理：删除 `.index` 即解除该目录的所有标签
- 零侵入：复用现有标签系统，无需额外代码

***

## 🤝 贡献指南

欢迎以 Issue / PR / 文档改进等任意形式贡献本项目。

### 工作流

1. Fork 本仓库
2. 创建功能分支：`git checkout -b feature/AmazingFeature`
3. 提交修改：`git commit -m 'feat: add AmazingFeature'`
4. 推送分支：`git push origin feature/AmazingFeature`
5. 发起 Pull Request

### 代码规范

提交前请确保以下命令均通过：

```bash
cargo fmt               # 代码格式化
cargo clippy            # 静态检查
cargo test              # 单元测试
cargo build --release   # 发布构建可通过
```

## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 发布，可自由使用、修改与分发。

## 📞 联系方式

- 项目主页：[rust-file-explorer (Gitee)](https://gitee.com/glacier-bot/rust-file-explorer)
- 问题反馈：[Gitee Issues](https://gitee.com/glacier-bot/rust-file-explorer/issues)
- 邮箱：<1098644849@qq.com>

***

⭐ 如果 rfe 帮到了你，欢迎点个 Star 支持一下！
