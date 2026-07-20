# rfe — Rust File Explorer

> 为终端设计的现代文件浏览器与工作流引擎

**高性能 · 跨平台 · 可组合**

[![Rust](https://img.shields.io/badge/Rust-1.65%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)](https://github.com/glacier-bot/rust-file-explorer)

---

## 目录

- [项目简介](#项目简介)
- [核心亮点](#核心亮点)
- [安装指南](#安装指南)
  - [环境要求](#环境要求)
  - [从源码编译](#从源码编译)
  - [接入系统 PATH](#接入系统-path)
  - [验证安装](#验证安装)
- [快速开始](#快速开始)
  - [启动方式](#启动方式)
  - [你的第一条命令](#你的第一条命令)
- [特色功能详解](#特色功能详解)
  - [行号导航（`-r`）](#行号导航-r)
  - [路径别名（`@`）](#路径别名-)
  - [命令链（`->`）](#命令链-)
  - [占位符与 POP 展开（`{}` / `{}.pop`）](#占位符与-pop-展开----pop)
  - [标签管理（`tag`）](#标签管理-tag)
  - [Shell 集成](#shell-集成)
- [目录历史（`cd -b`）](#目录历史-cd--b)
- [智能命令补全](#智能命令补全)
- [萌系模式（`-moe`）](#萌系模式--moe)
- [命令参考](#命令参考)
  - [目录浏览](#目录浏览)
  - [文件操作](#文件操作)
  - [剪贴板](#剪贴板)
  - [路径别名](#路径别名)
  - [标签管理](#标签管理-1)
  - [界面控制](#界面控制)
- [进阶用法](#进阶用法)
  - [别名与 POP 路径组合](#别名与-pop-路径组合)
  - [命令链实战](#命令链实战)
  - [标签目录约定（`.index`）](#标签目录约定index)
- [故障排除](#故障排除)
  - [安装与编译](#安装与编译)
  - [运行时问题](#运行时问题)
  - [快速诊断](#快速诊断)
- [技术栈](#技术栈)
- [贡献指南](#贡献指南)
- [许可证与联系](#许可证与联系)

---

## 项目简介

**rfe** 是一个使用 Rust 编写的高性能命令行文件浏览器。它在传统 `ls` / `cd` 的基础上，引入了一套**可组合的工作流语法**，包括行号导航、路径别名、命令链、占位符扩展等机制，让终端文件操作从"逐字输入"进化为"语义编排"。

无论是日常目录浏览、项目路径跳转，还是复杂的批量文件处理，rfe 都能通过简洁的表达式在单行内完成。

---

## 核心亮点

| 特性                         | 说明                                                 | 典型场景                                  |
| ---------------------------- | ---------------------------------------------------- | ----------------------------------------- |
| **行号导航（`-r`）**         | 基于 `ls` 输出序号快速操作文件，无需输入路径         | `cd -r 3` 直接跳转第 3 个目录             |
| **路径别名（`@`）**          | 为任意目录设置短名称，全局持久化                     | `@proj` 代替 `/home/user/projects/my-app` |
| **命令链（`->`）**           | 多命令串联，前序输出自动注入后续                     | `ls --re "\.rs$" -> open` 搜索并打开      |
| **占位符（`{}`）**           | 将前序输出插入任意参数位置                           | `cppwd -> alias add home {}`              |
| **POP 展开（`{}.pop`）**     | 在路径上向上回退 N 级目录                            | `cpf src/main.rs -> cd {}.pop` 跳到项目根 |
| **标签管理（`tag`）**        | 为文件打多标签，支持正则检索与批量筛选               | `tag find "rust\|work"` 跨目录检索        |
| **标签目录约定（`.index`）** | 借助 `.index` 文件让目录参与标签体系，实现按标签跳转 | `cd -idx work` 跳转到标签为 work 的目录   |
| **Shell 集成**               | 未识别命令自动转发系统 shell，目录变更自动同步       | `dir | findstr .rs -> echo Got: {}`       |
| **智能命令补全**             | 命令名、参数、子命令自动补全，两种模式配色区分明显   | 输入 `ls -` 按 `Tab` 查看所有参数选项     |
| **双运行模式**               | 交互式 REPL（补全 / 历史 / ESC 清空）与单次命令执行  | 连续操作用 REPL，脚本调用用单次           |

---

## 安装指南

### 环境要求

| 项目     | 要求                                         |
| -------- | -------------------------------------------- |
| Rust     | 1.65.0 及以上                                |
| Cargo    | 与 Rust 同步发布                             |
| 操作系统 | Windows 10+ / Linux（内核 4.15+）/ macOS 11+ |

### 从源码编译

```bash
git clone https://github.com/glacier-bot/rust-file-explorer.git
cd rust-file-explorer
cargo build --release
```

编译产物位于 `target/release/rfe`（Windows 下为 `rfe.exe`）。

### 接入系统 PATH

#### Linux / macOS

```bash
sudo cp target/release/rfe /usr/local/bin/
```

#### Windows（PowerShell）

**方式 A：复制到系统目录（需管理员权限）**

将 `target\release\rfe.exe` 复制到 `C:\Windows\System32` 或任意已加入 `PATH` 的目录。

**方式 B：将编译目录加入用户 PATH（推荐）**

```powershell
[Environment]::SetEnvironmentVariable(
    "Path",
    $env:Path + ";C:\path\to\rust-file-explorer\target\release",
    [EnvironmentVariableTarget]::User
)
```

执行后关闭并重新打开 PowerShell 窗口使其生效。

**方式 C：在 PowerShell 配置文件中设置别名**

```powershell
if (!(Test-Path -Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE -Force }
Add-Content $PROFILE 'Set-Alias -Name rfe -Value "C:\path\to\rust-file-explorer\target\release\rfe.exe"'
. $PROFILE
```

> **常见问题**：若 PowerShell 提示"无法加载文件，因为在此系统上禁止运行脚本"，请以管理员身份运行 `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser` 解除限制。

### 验证安装

```bash
rfe help
```

正常显示帮助信息即表示安装成功。

---

## 快速开始

### 启动方式

| 模式            | 命令                   | 适用场景                                   |
| --------------- | ---------------------- | ------------------------------------------ |
| **交互式 REPL** | `rfe`                  | 连续多次操作；享受补全、历史记录、ESC 清空 |
| **单次命令**    | `rfe <command> [args]` | 脚本调用、与其他 CLI 工具组合              |

> **提示**：REPL 模式下无需输入 `rfe` 前缀，直接键入命令即可。

### 你的第一条命令

```bash
# 进入 REPL
rfe

# 浏览当前目录
ls

# 使用行号快速跳转（假设第 2 项是 src 目录）
cd -r 2

# 返回上级
cd ..

# 退出 REPL
exit
```

---

## 特色功能详解

### 行号导航（`-r`）

每次执行 `ls` 后，条目会自动编号。使用 `-r <line>` 即可通过行号引用文件或目录，无需输入完整路径。

```bash
ls                    # 显示带行号的目录列表
cd -r 2              # 跳转到第 2 个条目对应的目录
open -r 3            # 用系统默认应用打开第 3 个条目
cpf -r 1             # 复制第 1 个条目的绝对路径到剪贴板
mv -r 2 -r 3 --cp    # 将第 2 个条目复制到第 3 个目录
```

**关键特性**

- **生命周期**：每次 `ls` 会更新行号记录，新记录完全替换旧记录
- **持久有效**：即使切换目录，之前的 `ls` 记录仍然有效（直到下次 `ls`）
- **子路径追加**：支持 `cd -r 2/src/main.rs` 形式深入子目录
- **分隔符兼容**：同时支持 `/` 与 `\`（Windows）
- **智能补全**：REPL 中输入 `cd -r 2/` 后按 `Tab` 可自动补全子路径
- **模式限制**：`-r` 仅在交互式 REPL 模式下可用

---

### 路径别名（`@`）

为常用目录设置短别名，之后用 `@别名` 即可瞬间定位。

```bash
# 管理别名
alias add proj /home/user/projects/my-app
alias remove proj
alias list

# 使用别名
cd @proj
ls @proj/src
open @dl/report.pdf
```

**关键特性**

- **全局可用**：兼容 `ls`、`cd`、`open`、`cpf`、`mv` 等所有路径相关命令
- **子路径拼接**：支持 `@别名/子路径` 形式深入目录
- **持久化存储**：自动保存至系统配置目录，重启后依旧有效
  - Windows：`%APPDATA%\rfe\aliases.json`
  - Linux / macOS：`~/.config/rfe/aliases.json`
- **交互式补全**：REPL 中输入 `@` + `Tab` 可自动补全已有别名

---

### 命令链（`->`）

通过 `->` 将多个命令串联，前一个命令的**原始输出**会自动作为输入传递给下一个命令，实现复杂操作的"一行式"表达。

```bash
cmd1 -> cmd2 -> cmd3    # 顺序执行；任一命令失败则中断
cmd1 ->! cmd2           # 容错节点：cmd1 失败也继续执行 cmd2
```

**典型示例**

```bash
# 浏览当前目录后跳到上级目录
pwd -> ls -> cd .. -> pwd

# 正则搜索 README 并复制其绝对路径到剪贴板
ls --re "^README\.md$" -> cpf

# 搜索并打开 .rs 文件
ls --re "\.rs$" -> open

# 容错示例：目录不存在也继续执行
rfe cd maybe_nonexist ->! ls
```

**关键特性**

- 顺序执行 + 数据管道：前序原始输出自动作为下一命令的首参数
- 双重错误策略：`->` 严格中断 / `->!` 容错继续
- 兼容所有 rfe 命令，REPL 与直接模式均可用

---

### 占位符与 POP 展开（`{}` / `{}.pop`）

默认情况下，前序输出作为下一命令的首参数。使用 `{}` 可将其插入到**任意位置**、**多次引用**，或**拼接子路径**。

```bash
cmd1 -> cmd2 arg1 {} arg3     # 插入到中间位置
cmd1 -> cmd2 {} {}            # 多次引用同一输出
cmd1 -> cmd2 {}/subpath       # 拼接子路径
```

在占位符后追加 `.pop`（或简写 `.`），表示在路径上**向上回退对应层级**。每多一个 `.pop` / `.` 即向上一级。

```bash
cmd -> cd {}.pop          # 上一级（父目录），等价于 cmd -> cd {}.
cmd -> cd {}.pop.pop      # 上两级，等价于 cmd -> cd {}..
cmd -> cd {}...           # 上三级（简写形式）
```

> **提示**：`.pop` 是语义化写法（更易读），`.` 是简写形式（更紧凑），两者可混用，例如 `{}.pop.` 等价于上两级。

**典型示例**

```bash
# 用当前路径快速添加别名（输出位于命令尾部）
cppwd -> alias add desktop {}

# 复制文件路径，并为该文件添加多个标签
cpf main.rs -> tag add {} rust code

# 打开前一个目录下的特定文件
pwd -> open {}/test.txt

# 使用路径弹出后再拼接子路径
cpf src/main.rs -> open {}.pop/test.txt

# 从深层文件快速跳到项目根
cpf src/utils/mod.rs -> cd {}.pop.pop.pop

# 取当前目录的祖父目录并设为别名
pwd -> alias add ancestor {}.pop.pop
```

**关键特性**

- 精准定位：可在命令中任意位置插入
- 多重引用：同一输出可在多处被替换
- 子路径拼接：支持 `{}/subpath` 或 `{}\subpath` 形式深入目录
- 任意级联：POP 回退理论上无层级上限
- 边界安全：超出实际层级时自动停在最顶层并友好提示
- 跨平台：兼容 Windows 与 Unix 路径

---

### 标签管理（`tag`）

为文件添加多标签，支持正则查询、批量筛选与自动备份。

```bash
# 管理标签
tag add <file> <tag1> [tag2...]       # 添加标签
tag remove <file> <tag1> [tag2...]    # 删除标签
tag clear <file>                      # 清空全部标签
tag get <file>                        # 查看标签
tag list                              # 列出所有带标签的文件
tag find <pattern1> [pattern2...]     # 全局按标签搜索（正则、多条件组合）
tag backup / tag restore              # 手动备份 / 恢复
```

> **提示**：`tag` 命令支持简写 `t`，例如 `t add main.rs rust`。部分子命令额外接受常见别名：`tag remove` ≡ `tag rm`、`tag list` ≡ `tag ls`、`tag find` ≡ `tag search`。

**关键特性**

- 多标签支持：单个文件可拥有任意数量标签
- 正则检索：`tag find` 支持完整正则语法，可多条件组合
- 自动备份：每次标签修改自动生成 `.bak`，防止误操作丢失数据
- 持久化存储：与别名共用配置目录

---

### Shell 集成

rfe 能够**无缝桥接系统 shell 能力**：凡未被识别为 rfe 内置命令的输入，均自动转发到系统 shell（Windows 为 PowerShell / cmd，Linux / macOS 为 `sh`）执行。更重要的是，**shell 中的目录变更会自动同步到 rfe 主进程**。

```bash
# 直接运行系统命令
dir
Get-ChildItem
ls -la

# shell 管道与命令链可混用
dir | findstr .rs -> echo Found: {}

# shell 中的 cd 会自动同步到 rfe
cd src; pwd       # 进入 src，rfe 工作目录同步变更
pushd ..; pwd     # pushd / popd 同样支持
popd
```

**关键特性**

- **零配置自动转发**：无需前缀，未知命令直接交给 shell
- **目录双向同步**：`cd` / `chdir` / `pushd` / `popd` 等目录操作结果自动同步到 rfe
- **完整管道支持**：shell 管道 `|` 与 rfe 命令链 `->` 可混合使用
- **跨平台适配**：Windows PowerShell / cmd 与 Unix shell 自动识别

> **PowerShell 提示**：PowerShell 的 `echo` 会将空格分隔的参数逐行输出，这是 PowerShell 本身的特性，不是 rfe 的问题。如需保持单行输出，请使用引号包裹：`echo "Got: {}"`。

---

### 目录历史（`cd -b`）

无需记忆路径即可在两个工作目录间快速来回切换。

```bash
cd /path/to/dir1
cd /path/to/dir2
cd -b       # 返回 dir1
cd -back    # 再次返回 dir2
```

- `-b` 与 `-back` 等价
- 跳转后自动更新历史，错误时友好提示
- 完全兼容 `cd ..`、`cd ~` 及命令链场景

---

### 智能命令补全

REPL 模式下提供全面的智能补全支持，涵盖命令名、参数、子命令与标签，配合输入提示大幅减少记忆负担。

```bash
# 命令名补全：输入前缀按 Tab 自动列出匹配命令
l<Tab>    # → ls
c<Tab>    # → cd / cppwd / clear / change

# 参数补全：输入命令后按 - 再按 Tab 查看所有可用选项
ls -<Tab>     # → -a -l -la -t --re --tags ...
cd -<Tab>     # → -b -back -r -idx ...

# 子命令补全
tag a<Tab>    # → tag add
alias l<Tab>  # → alias list

# 标签补全
tag add file.rs ru<Tab>  # 自动补全已有标签
```

**关键特性**

- **多维度补全**：命令名、短/长参数、子命令、标签全覆盖
- **右方向键接受**：光标在行尾时按 `→` 直接采用提示内容
- **配色差异**：标准模式与萌系模式使用完全不同的调色板
  - **Std**：绿/蓝/黄/青，简洁专业
  - **Moe**：热粉/紫/橙/浅粉，风格鲜明统一
- **零配置**：开箱即用，无需额外设置

---

### 萌系模式（`-moe`）

启用后所有输出统一切换为粉色系配色（RGB: 255, 105, 180）并附加场景化颜文字，让终端操作更治愈。**命令补全列表也会同步切换为萌系配色。**

```bash
rfe -moe              # 以萌系模式进入 REPL
rfe -moe <command>    # 以萌系模式执行单条命令
change -moe           # REPL 中动态切换
change -std           # 切回标准模式
```

- 全命令覆盖，纯视觉增强，不影响任何原有功能
- 运行时可动态切换，无需重启

---

## 命令参考

### 目录浏览

| 命令                                     | 说明                                  |
| ---------------------------------------- | ------------------------------------- |
| `ls`                                     | 列出当前目录内容（带行号）            |
| `ls <path>`                              | 列出指定目录内容                      |
| `ls -a`                                  | 同时显示隐藏文件                      |
| `ls -l`                                  | 显示详细信息（大小、创建 / 修改时间） |
| `ls -la`                                 | 等同 `-a -l`                          |
| `ls --re <pattern>`                      | 正则全局搜索（当前目录）              |
| `ls --re-deep <pattern>`                 | 递归正则搜索                          |
| `ls --re --xcaps <pattern>`              | 大小写不敏感正则搜索                  |
| `ls -tag` / `ls --tags`                  | 列表中附带显示标签                    |
| `ls -t <pattern>` / `ls --tag <pattern>` | 按标签过滤，可重复传参组合多标签      |
| `ls -t --deep <pattern>`                 | 递归按标签过滤                        |

### 文件操作

| 命令                   | 说明                                             |
| ---------------------- | ------------------------------------------------ |
| `mv <src> <dest>`      | 移动文件 / 目录                                  |
| `mv <src> <dest> --cp` | 复制文件 / 目录（保留原文件）                    |
| `open <path>`          | 用系统默认应用打开文件，或在资源管理器中打开目录 |
| `open -r <line>`       | 按 `ls` 输出的行号打开文件 / 目录                |
| `open -tag <pattern>`  | 按标签匹配打开对应目录（`.index` 约定）          |
| `mkdf -f <path>`       | 创建文件，自动补齐父目录                         |
| `mkdf -d <path>`       | 创建文件夹                                       |
| `mkdf -d -p <path>`    | 创建多级嵌套文件夹                               |

### 剪贴板

| 命令         | 说明                               |
| ------------ | ---------------------------------- |
| `cppwd`      | 复制当前目录绝对路径到系统剪贴板   |
| `cpf <file>` | 复制指定文件的绝对路径到系统剪贴板 |

### 路径别名

| 命令                      | 说明                     |
| ------------------------- | ------------------------ |
| `alias add <name> <path>` | 添加 / 更新别名          |
| `alias remove <name>`     | 删除别名                 |
| `alias list`              | 查看全部别名             |
| `@<name>[/subpath]`       | 在任意路径参数中引用别名 |

> `alias add` ≡ `alias set`、`alias remove` ≡ `alias rm` ≡ `alias delete`、`alias list` ≡ `alias ls`

### 标签管理

| 命令                                 | 说明                 |
| ------------------------------------ | -------------------- |
| `tag add <file> <tag1> [tag2...]`    | 添加标签             |
| `tag remove <file> <tag1> [tag2...]` | 删除标签             |
| `tag clear <file>`                   | 清空全部标签         |
| `tag get <file>`                     | 查看标签             |
| `tag list`                           | 列出所有带标签的文件 |
| `tag find <pattern1> [pattern2...]`  | 全局按标签搜索       |
| `tag backup` / `tag restore`         | 手动备份 / 恢复      |

### 界面控制

| 命令                           | 说明                                 |
| ------------------------------ | ------------------------------------ |
| `welcome`                      | 重新显示欢迎页面（自动适配当前模式） |
| `clear` / `cls`                | 清空终端屏幕                         |
| `help` / `?`                   | 显示帮助信息                         |
| `change -std` / `change --std` | 切换为标准模式                       |
| `change -moe` / `change --moe` | 切换为萌系模式                       |
| `ESC`（按键）                  | REPL 模式下清空当前输入行            |
| `exit` / `quit` / `q`          | 退出 REPL                            |

---

## 进阶用法

### 别名与 POP 路径组合

```bash
# 快速将当前目录的父目录设为别名
pwd -> alias add proj {}.pop

# 先复制深层文件路径，再向上回退两级打开同目录下的配置
cpf src/utils/mod.rs -> open {}..

# 正则搜索项目根目录的 README，复制其路径并跳转到文件所在目录
ls --re "^README\.md$" -> cpf -> cd {}.pop
```

### 命令链实战

```bash
# 链式容错：尝试进入项目目录，失败则进入别名备份目录
cd ./maybe_proj ->! cd @proj -> ls

# 搜索标签为 config 的文件并直接打开
tag find "^config$" -> open {}

# 复制当前路径、添加别名、再列出别名目录内容
cppwd -> alias add current {} -> ls @current
```

### 标签目录约定（`.index`）

rfe 的标签系统作用于**文件**而非目录。借助 `.index` 文件约定，可让目录也参与标签体系，并配合 `cd -idx` 实现按标签跳转。

```bash
# 1. 在目标目录下创建 .index 占位文件
mkdf -f /path/to/folder/.index

# 2. 为 .index 文件打标签
tag add /path/to/folder/.index work project important

# 3. 按标签跳转 / 检索
cd -idx work               # 直接跳转
open -tag work             # 直接在资源管理器中打开
tag find work project      # 全局搜索匹配的目录
ls -t work                 # 当前目录下按标签过滤
```

**使用建议**

- **全局 / 磁盘级路径**：优先使用 `alias`。别名全局持久化，适合高频、跨项目的通用路径。
- **项目 / 文件夹级路径**：优先使用 `.index` 标签约定。标签与目录绑定，随项目移动 / 删除自动生效 / 失效，更适合项目级别的路径管理。
- **隐藏属性**：创建 `.index` 后建议设置隐藏属性，避免干扰文件列表：
  - Windows：`attrib +h .index`
  - Linux / macOS：以 `.` 开头的文件默认隐藏，无需额外操作

---

## 故障排除

### 安装与编译

| 问题                                         | 可能原因              | 解决方案                                                                          |
| -------------------------------------------- | --------------------- | --------------------------------------------------------------------------------- |
| `cargo build --release` 失败                 | Rust 版本过低         | 执行 `rustup update`，确保 >= 1.65.0                                              |
|                                              | 缺少系统依赖（Linux） | 安装 `build-essential` / `gcc` / `clang`                                          |
| PowerShell 提示"无法加载文件...禁止运行脚本" | 执行策略受限          | 管理员运行 `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser` |
| 加入 PATH 后仍无法识别 `rfe`                 | 未重启终端 / 路径错误 | 关闭并重新打开终端；检查路径是否包含 `rfe.exe` 实际目录                           |

### 运行时问题

| 问题                            | 可能原因                     | 解决方案                                                         |
| ------------------------------- | ---------------------------- | ---------------------------------------------------------------- |
| `-r` 参数无效或提示"行号不存在" | 未先执行 `ls` / 行号已过期   | 先执行 `ls` 查看最新行号；`-r` 仅在 REPL 模式下可用              |
| `cd -idx <tag>` 无反应或报错    | 目标目录不存在 `.index` 文件 | 在目标目录下创建 `.index` 文件并为其添加标签                     |
| 别名 `@<name>` 无法解析         | 别名未添加 / 拼写错误        | 使用 `alias list` 检查已有别名；注意区分大小写                   |
| 标签数据丢失                    | 异常退出 / 手动误删          | 检查配置目录下的 `.bak` 备份文件，执行 `tag restore` 恢复        |
| 萌系模式 `-moe` 无颜色输出      | 终端不支持 ANSI 颜色         | 更换终端（Windows Terminal、iTerm2、GNOME Terminal 等）          |
| 剪贴板操作 `cpf` / `cppwd` 失败 | 无图形会话 / 远程 SSH        | 在本地桌面会话中运行；Linux 可尝试安装 `xclip` 或 `wl-clipboard` |
| 命令链 `->` 中断                | 前序命令返回非零退出码       | 若希望容错，使用 `->!` 代替 `->`                                 |

### 快速诊断

1. **确认版本与环境**

   ```bash
   rfe help          # 确认 rfe 可被调用
   rustc --version   # 确认 Rust 版本 >= 1.65.0
   ```

2. **检查配置文件完整性**

   ```bash
   # Windows
   ls "$env:APPDATA\rfe\"
   # Linux / macOS
   ls ~/.config/rfe/
   ```

   确认 `aliases.json` 与 `tags.json` 格式正确，必要时从 `.bak` 恢复。

3. **查看详细错误信息**
   - REPL 模式下命令会直接回显错误原因
   - 直接执行模式下，使用 `rfe <command>` 观察标准错误输出

4. **清理与重置**
   ```bash
   # 备份后删除配置文件，可强制重置所有别名与标签
   mv ~/.config/rfe/aliases.json ~/.config/rfe/aliases.json.bak
   mv ~/.config/rfe/tags.json ~/.config/rfe/tags.json.bak
   ```

若以上方案无法解决，欢迎通过 [Gitee Issues](https://gitee.com/glacier-bot/rust-file-explorer/issues) 提交问题，附上操作系统版本、Rust 版本及复现步骤。

---

## 技术栈

### 核心依赖

| 依赖                   | 版本 | 用途                         |
| ---------------------- | ---- | ---------------------------- |
| `colored`              | 2.1  | 终端彩色输出                 |
| `crossterm`            | 0.28 | 跨平台终端按键事件           |
| `rustyline`            | 12.0 | REPL 行编辑、补全与历史记录  |
| `arboard`              | 3.4  | 跨平台剪贴板访问             |
| `regex`                | 1.10 | 正则搜索与标签匹配           |
| `open`                 | 5.0  | 调用系统默认程序打开文件     |
| `dirs`                 | 5.0  | 获取跨平台配置 / 主目录路径  |
| `unicode-width`        | 0.1  | Unicode 字符宽度计算         |
| `serde` / `serde_json` | 1.0  | 别名、标签数据的 JSON 持久化 |
| `tempfile`             | 3.10 | 标签备份等场景的临时文件管理 |

### 配置文件位置

别名与标签数据以 JSON 形式持久化到系统配置目录，重启后保留。每次标签修改自动生成 `.bak` 备份。

| 平台          | 路径                                                                 |
| ------------- | -------------------------------------------------------------------- |
| Windows       | `%APPDATA%\rfe\aliases.json`、`%APPDATA%\rfe\tags.json`（含 `.bak`） |
| Linux / macOS | `~/.config/rfe/aliases.json`、`~/.config/rfe/tags.json`（含 `.bak`） |

---

## 贡献指南

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

---

## 许可证与联系

- **许可证**：[MIT 许可证](LICENSE)
- **项目主页**：[rust-file-explorer (Gitee)](https://gitee.com/glacier-bot/rust-file-explorer)
- **问题反馈**：[Gitee Issues](https://gitee.com/glacier-bot/rust-file-explorer/issues)
- **邮箱**：<1098644849@qq.com>

---

如果 rfe 帮到了你，欢迎点个 Star 支持一下！
