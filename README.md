# Rust File Explorer (rfe)

一个高性能、跨平台的命令行文件浏览器，使用 Rust 编写，提供直观的彩色界面、文件类型图标和丰富的文件操作功能，让你在终端中也能高效管理文件。

## ✨ 核心功能

- 🎨 **彩色终端输出**：支持不同文件类型的彩色标识和 emoji 图标
- 📁 **目录浏览**：支持显示隐藏文件、详细文件信息（大小、创建时间、修改时间）
- 📋 **剪贴板集成**：一键复制当前目录路径或文件绝对路径到系统剪贴板
- 🚀 **跨平台兼容**：完美支持 Windows、Linux、macOS 三大操作系统
- ⌨️ **智能交互**：命令自动补全、历史记录功能，提升操作效率
- 📂 **文件操作**：支持文件/目录移动、复制、使用系统默认应用打开
- 🎯 **双运行模式**：支持交互式 REPL 模式和直接命令执行模式
- ⚡ **轻量高性能**：Rust 编译的原生二进制，启动速度快，资源占用低
- 🔍 **正则搜索**：支持使用正则表达式全局搜索文件和目录，支持大小写敏感/不敏感匹配
- 🔖 **路径别名**：支持自定义目录别名，使用`@别名`快速访问常用目录，无需输入冗长路径
- 🏷️ **标签管理**：支持为文件/文件夹添加自定义标签，按标签筛选搜索文件，提升文件管理效率
- 🔗 **命令链式执行**：使用 `->` 将多个命令串联执行，支持前一个命令的输出作为后一个命令的输入，配合 `->!` 实现错误跳过机制
- 📍 **`{}`** **占位符扩展**：在命令链中使用 `{}` 将前一个命令的输出精准插入到任意位置，实现更灵活的参数传递
- 🆕 **`mkdf`** **命令**：支持同时创建文件和文件夹，提供便捷的创建体验
- 🆕 **ESC 键清空输入**：在 REPL 模式下，按 ESC 键可快速清空当前输入内容，提高操作效率
- 🔙 **目录快速返回**：使用 `cd -b` 或 `cd -back` 快速返回上一个工作目录，无需记住路径
- 🌸 **Moe Moe 模式**：使用 `-moe` 或 `--moe` 参数启动，欢迎界面和所有输出增加可爱颜文字表情，配色统一为粉色系，更适合二次元宝宝体质

## 🔑 核心概念与进阶特性

本节详细介绍 rfe 的两项核心语法特性：**`@`** **路径别名** 与 **`->`** **命令链式执行**。理解这两个概念将显著提升你在终端中的文件操作效率。

***

### 🔖 `@` — 路径别名（Path Alias）

**路径别名** 是 rfe 提供的一种快捷路径引用机制。你可以为常用目录设置一个简短的别名，之后在任何需要输入路径的地方，使用 `@别名` 即可瞬间定位到该目录，无需记忆或输入冗长的绝对路径。

#### 基本语法

```bash
@<alias>              # 直接使用别名替换为完整路径
@<alias>/<subpath>    # 别名后接子路径，实现快速深入目录
```

#### 管理别名

| 命令                        | 说明      | 示例                               |
| ------------------------- | ------- | -------------------------------- |
| `alias add <name> <path>` | 添加/更新别名 | `rfe alias add docs ~/Documents` |
| `alias remove <name>`     | 删除别名    | `rfe alias remove docs`          |
| `alias list`              | 列出所有别名  | `rfe alias list`                 |

#### 使用示例

```bash
# 添加常用目录别名
rfe alias add proj ~/projects/rust-project
rfe alias add dl ~/Downloads

# 使用别名进行目录跳转
rfe cd @proj
rfe ls @proj/src
rfe open @dl/report.pdf

# 别名拼接子路径
rfe ls @proj/src/components
rfe mv temp.txt @dl/archive/
```

#### 核心特性

- **全局可用**：`@别名` 可在任何接收路径参数的命令中使用，包括 `ls`、`cd`、`open`、`cpf`、`mv` 等
- **子路径拼接**：支持在别名后直接追加子路径，如 `@proj/src/main.rs`
- **持久化存储**：别名数据自动保存至系统配置目录，重启后依然有效
- **交互式补全**：在 REPL 模式下输入 `@` 后按 Tab，可自动补全已有别名并显示对应路径
- **跨平台兼容**：配置存储位置
  - Windows: `%APPDATA%\rfe\aliases.json`
  - Linux/macOS: `~/.config/rfe/aliases.json`

#### 适用场景

- **项目开发**：为频繁访问的项目根目录设置别名，快速在源码、测试、构建目录间切换
- **资源目录**：为 Downloads、Documents、Desktop 等常用目录设置短别名
- **深路径导航**：避免反复输入深层嵌套的路径（如 `~/company/team/project/module/src`）

***

### 🔗 `->` — 命令链式执行（Command Chain）

**命令链式执行** 是 rfe 独有的工作流特性，允许你将多个命令通过 `->` 连接成一条执行链。链中的命令按顺序执行，前一个命令的**原始输出**会自动作为输入数据传递给后一个命令，从而实现复杂的多步文件操作。

#### 基本语法

```bash
cmd1 -> cmd2 -> cmd3        # 顺序执行，任一命令失败则中断并返回错误
cmd1 ->! cmd2 -> cmd3       # 使用 ->! 表示即使前一个命令失败，也继续执行后续命令
```

#### 命令间的数据传递

链式执行时，前一个命令的**原始路径输出**（raw output）会被注入到后一个命令中：

- `pwd` 的原始输出是当前目录的绝对路径，可作为 `ls` 或 `cd` 的输入
- `ls --re <pattern>` 的原始输出是匹配文件/目录的路径，可作为 `cpf`、`open`、`cd` 的输入
- `mv` 的原始输出是目标路径，可继续传递给下游命令

#### 使用示例

```bash
# 示例 1：查看当前目录，列出内容，返回上级，再次查看路径
rfe pwd -> ls -> cd .. -> pwd

# 示例 2：正则搜索单个文件，并将其绝对路径复制到剪贴板
rfe ls --re "^README\\.md$" -> cpf

# 示例 3：搜索 .rs 文件，打开找到的第一个文件
rfe ls --re "\\.rs$" -> open

# 示例 4：链式导航——进入项目目录，列出源码，返回
rfe cd @proj -> ls src -> cd ..

# 示例 5：使用 ->! 跳过可能的错误，确保后续命令执行
rfe cd maybe_nonexist_dir ->! ls
```

#### 核心特性

- **顺序执行**：命令按从左到右的顺序依次执行，逻辑清晰可控
- **数据管道**：前一个命令的原始输出自动成为下一个命令的输入参数，无需手动复制粘贴路径
- **错误控制**：默认模式下，任一命令失败会中断整个链条并报错；使用 `->!` 可标记**容错节点**，即使该命令失败也继续执行后续命令
- **兼容所有命令**：`->` 可连接 `ls`、`cd`、`pwd`、`cpf`、`open`、`mv`、`clear` 等所有 rfe 命令
- **REPL 与直接模式均支持**：无论在交互式会话还是单行命令中均可使用

#### 适用场景

- **文件定位与操作**：先搜索文件，再对搜索结果直接进行复制、打开、移动等操作
- **批量导航**：在多个相关目录间快速跳转并执行查看命令
- **脚本化工作流**：将日常重复的多步操作浓缩为一条命令链，减少键盘输入
- **容错脚本**：在不确定某个目录或文件是否存在时，使用 `->!` 确保后续关键步骤依然执行

***

### 📍 `{}` — 占位符扩展（Placeholder Expansion）

在命令链中，默认情况下前一个命令的输出会作为**整体输入**传递给下一个命令的**第一个参数位置**。但在某些场景下，你可能需要将前一个命令的输出插入到**任意位置**，或**多次使用**。此时可以使用 `{}` 作为占位符，rfe 会自动将其替换为前一个命令的原始输出。

#### 基本语法

```bash
cmd1 -> cmd2 <arg1> {} <arg3>     # 将 cmd1 的输出替换到 {} 位置
cmd1 -> cmd2 {} {}                # 多个 {} 都会被替换为同一个输出
```

#### 使用示例

```bash
# 示例 1：获取当前路径，并将路径作为参数添加别名
rfe cppwd -> alias add desktop {}

# 示例 2：获取当前路径，复制路径后列出该路径的内容
rfe cppwd -> ls {}

# 示例 3：复制文件路径到剪贴板，然后为该文件添加标签
rfe cpf main.rs -> tag add {} rust code

# 示例 4：结合 ->! 使用，即使命令失败也能传递输出
rfe cd maybe_nonexist ->! cppwd -> alias add fallback {}
```

#### 核心特性

- **精准定位**：前一个命令的输出会被替换到 `{}` 出现的任意位置，不再局限于第一个参数
- **多占位符替换**：一条命令中可以出现多个 `{}`，所有占位符都会被替换为相同的输出内容
- **与默认传递共存**：即使命令中没有 `{}`，前一个命令的输出仍然会自动作为输入传递
- **支持所有输出命令**：`pwd`、`cppwd`、`cpf`、`ls --re`、`mv` 等命令的输出均可通过 `{}` 传递给后续命令

#### 适用场景

- **参数位于中间**：当需要将输出插入到命令的中间参数位置时（如 `alias add 名称 {}`）
- **多次使用同一输出**：当一条命令中需要多次引用前一个输出时
- **复杂命令构造**：需要精确控制输出在命令中的位置，而非默认的第一个参数位置

***

### 📦 `mkdf` — 多功能文件/文件夹创建命令

**mkdf命令** 是一个全能的创建工具，支持同时创建文件和文件夹，并提供便捷的路径处理功能。

#### 基本语法

```bash
mkdf -f <path>               # 创建文件
mkdf -d <path>               # 创建文件夹
mkdf -d -p <path>            # 创建文件夹，自动创建缺失的父目录
mkdf -h / --help            # 查看帮助信息
```

#### 使用示例

```bash
# 示例 1：创建一个简单的文件
mkdf -f test.txt

# 示例 2：创建文件并自动创建父目录
mkdf -f path/to/nested/file.txt

# 示例 3：创建文件夹
mkdf -d new_folder

# 示例 4：创建嵌套文件夹结构
mkdf -d -p parent/child/grandchild

# 示例 5：使用mkdf --help查看完整帮助
mkdf --help
```

#### 核心特性

- **文件/文件夹二合一**：使用 `-f` 指定创建文件，`-d` 指定创建文件夹
- **自动父目录创建**：创建文件时自动创建所有必需的父目录，无需手动提前创建
- **支持-p参数**：创建文件夹时使用 `-p` 可自动创建多级父目录
- **友好错误提示**：若路径已存在或有权限问题，提供清晰的错误信息
- **路径灵活处理**：支持绝对路径、相对路径和别名路径

#### 适用场景

- **快速创建单个文件/文件夹**：避免使用多条命令
- **创建嵌套目录结构**：一次性创建完整的目录树，无需手动逐层创建
- **配合其他命令使用**：与命令链结合使用，实现复杂的文件创建流程

***

### ⌨️ ESC 键清空输入 — REPL 模式便捷操作

在 REPL 交互式模式下，rfe 提供了一个实用的快捷键功能：按 ESC 键可以快速清空当前正在输入的命令行内容，无需逐个字符删除。

#### 功能说明

- **触发方式**：在输入命令过程中按下键盘 ESC 键
- **行为**：当前输入行内容被完全清空，不触发任何命令执行
- **效果**：输入区域恢复为空白状态，可重新输入新命令
- **跨平台兼容**：支持 Windows、Linux、macOS 系统上的各类终端模拟器

#### 使用场景

- **误输入内容**：快速清除错误的输入，重新开始输入
- **输入内容过长**：一次性清空，无需逐字符退格
- **改变主意**：决定不执行当前输入的命令，快速清空

***

### 🔙 `cd -b/-back` — 快速返回上一个目录

**目录快速返回** 功能让你在切换多个目录后，无需记住路径即可快速返回上一个工作目录，极大提升目录导航效率。

#### 基本语法

```bash
cd -b        # 短形式，快速返回上一个目录
cd -back     # 长形式，完整的返回命令
```

#### 使用示例

```bash
# 示例 1：在两个目录间来回切换
cd /path/to/directory1
cd /path/to/directory2
cd -b        # 返回 /path/to/directory1
cd -back     # 再次返回 /path/to/directory2

# 示例 2：连续切换后返回
cd ~/projects
cd src
cd components
cd -b        # 返回 src
cd -b        # 返回 ~/projects

# 示例 3：与命令链结合使用
cd @proj -> ls src -> cd -b -> pwd
```

#### 核心特性

- **两种参数形式**：支持短形式 `-b` 和长形式 `-back`，满足不同输入习惯
- **自动历史记录**：每次成功的目录切换都会自动保存上一个目录
- **友好错误提示**：在没有历史记录时会给出清晰的错误提示
- **完整兼容性**：不影响现有的 `cd ~`、`cd ..`、`cd <path>` 等功能
- **命令链支持**：在 `->` 命令链中也能正常工作，保存状态正确传递
- **REPL 与直接模式均支持**：无论在交互式会话还是单行命令中均可使用

#### 适用场景

- **快速目录切换**：在两个相关目录间频繁跳转，如项目目录和构建输出目录
- **临时访问目录**：快速访问某个目录后立即返回原工作目录
- **多层导航后返回**：在深层目录中操作后，快速返回到起点
- **命令链导航**：配合 `->` 命令链，实现复杂的目录浏览流程

***

### 🌸 `-moe` — Moe Moe 模式

**Moe Moe 模式** 为 rfe 带来可爱的萌系风格！启用后所有的输出都会增加符合内容的颜文字表情，配色也会统一改为粉色系，让你的终端操作更加治愈～ 💕

#### 基本语法

```bash
rfe -moe              # 以 Moe 模式进入 REPL 交互模式
rfe --moe             # 长形式参数
rfe -moe <command>    # 以 Moe 模式执行命令
```

#### 使用示例

```bash
# 示例 1：进入 Moe 模式的交互界面
rfe -moe
# 欢迎界面将显示：ciallo∠・ω⌒☆ Welcome to the moe moe mode！

# 示例 2：以 Moe 模式列出文件
rfe -moe ls

# 示例 3：以 Moe 模式查看当前路径
rfe -moe pwd

# 示例 4：以 Moe 模式查看帮助
rfe -moe help
```

#### 核心特性

- **粉色系配色**：所有输出使用统一的粉色系配色（RGB: 255, 105, 180），温暖可爱
- **智能颜文字**：每条输出都会根据内容添加合适的颜文字表情（💖, 🌸, ✨, 😢, 👋 等）
- **特殊欢迎语**：`ciallo∠・ω⌒☆ Welcome to the moe moe mode！` 萌系欢迎信息
- **完整支持**：所有命令均支持 Moe 模式，包括 REPL 交互和直接命令执行
- **完全兼容**：不影响任何原有功能和命令参数，纯视觉体验升级

#### 适用场景

- **日常使用**：让单调的终端操作变得生动有趣
- **心情调节**：粉粉的界面和可爱的颜文字能带来好心情
- **个性化**：展示你的独特终端风格
- **演示展示**：在演示或教学中使用，增加趣味性

***

## 🛠️ 技术栈

| 技术/依赖         | 版本           | 用途                     |
| ------------- | ------------ | ---------------------- |
| Rust          | 2021 Edition | 核心开发语言                 |
| colored       | 2.1          | 终端彩色输出                 |
| dirs          | 5.0          | 跨平台系统目录路径获取            |
| unicode-width | 0.1          | Unicode 字符串宽度计算，优化界面排版 |
| open          | 5.0          | 调用系统默认应用打开文件           |
| rustyline     | 12.0         | 命令行交互框架，支持补全、历史记录      |
| arboard       | 3.4          | 跨平台剪贴板操作               |
| regex         | 1.10         | 正则表达式搜索支持              |
| serde         | 1.0          | 序列化/反序列化支持             |
| serde\_json   | 1.0          | JSON格式数据持久化            |

## 📋 环境要求

- Rust 1.65.0 或更高版本
- Cargo 包管理器
- 支持的操作系统：
  - Windows 10+
  - Linux (内核版本 4.15+)
  - macOS 11+ (Big Sur)

## 📦 安装步骤

### 从源码编译安装

1. 克隆项目仓库：

```bash
git clone https://github.com/yourusername/rust-file-explorer.git
```

1. 进入项目目录：

```bash
cd rust-file-explorer
```

1. 编译发布版本：

```bash
cargo build --release
```

1. 安装二进制文件到系统路径：

- **Windows**：
  复制 `target\release\rfe.exe` 到 `C:\Windows\System32` 或其他已加入 `PATH` 的目录。
- **Linux/macOS**：

```bash
sudo cp target/release/rfe /usr/local/bin/
```

1. 验证安装：

```bash
rfe help
```

## 🚀 使用指南

### 两种运行模式

#### 1. 交互式 REPL 模式

直接运行 `rfe` 进入交互模式，界面会显示欢迎信息和命令提示，支持所有文件操作命令，具备命令补全和历史记录功能。

```bash
rfe
```

#### 2. 直接命令执行模式

不需要进入交互界面，直接在终端中执行命令：

```bash
rfe <command> [arguments]
```

### 完整命令列表

| 命令                                          | 说明                          | 示例                                                        |
| ------------------------------------------- | --------------------------- | --------------------------------------------------------- |
| `ls`                                        | 列出当前目录内容                    | `rfe ls`                                                  |
| `ls -l`                                     | 列出目录详细信息（包含大小、创建时间、修改时间）    | `rfe ls -l`                                               |
| `ls -a`                                     | 列出所有文件，包含隐藏文件               | `rfe ls -a`                                               |
| `ls -la`                                    | 列出所有文件的详细信息                 | `rfe ls -la`                                              |
| `ls <path>`                                 | 列出指定目录的内容                   | `rfe ls ~/Documents`                                      |
| `ls --re <pattern>`                         | 使用正则表达式全局搜索文件/目录            | `rfe ls --re \.rs$`（搜索所有.rs文件）                            |
| `ls --re --re-insensitive <pattern>`        | 大小写不敏感的正则搜索                 | `rfe ls --re --re-insensitive cargo`（搜索包含cargo的文件，不区分大小写） |
| `pwd`                                       | 打印当前工作目录路径                  | `rfe pwd`                                                 |
| `cppwd`                                     | 复制当前目录路径到剪贴板                | `rfe cppwd`                                               |
| `cpf <file>`                                | 复制指定文件的绝对路径到剪贴板             | `rfe cpf README.md`                                       |
| `cd` / `cd ~`                               | 切换到用户主目录                    | `rfe cd`                                                  |
| `cd ..`                                     | 切换到上级目录                     | `rfe cd ..`                                               |
| `cd <path>`                                 | 切换到指定目录                     | `rfe cd /usr/local/bin`                                   |
| `cd -b` / `cd -back`                        | 快速返回上一个工作目录                 | `rfe cd -b`、`rfe cd -back`                                |
| `open <path>`                               | 使用系统默认应用打开文件 / 在资源管理器中打开文件夹 | `rfe open document.pdf` / `rfe open ~/Documents`          |
| `mv <source> <dest>`                        | 移动文件/目录到目标位置                | `rfe mv file.txt ~/Documents/`                            |
| `mv <source> <dest> --cp`                   | 复制文件/目录到目标位置（保留原文件）         | `rfe mv photo.jpg ~/Pictures/ --cp`                       |
| `clear` / `cls`                             | 清空终端屏幕                      | `rfe clear`                                               |
| `help` / `?`                                | 显示帮助信息                      | `rfe help`                                                |
| `alias add <name> <path>`                   | 添加路径别名                      | `rfe alias add docs ~/Documents`                          |
| `alias remove <name>`                       | 删除路径别名                      | `rfe alias remove docs`                                   |
| `alias list`                                | 查看所有路径别名                    | `rfe alias list`                                          |
| `@<alias>`                                  | 使用路径别名，可用于所有需要路径的命令         | `rfe ls @docs`, `rfe cd @docs/rust`                       |
| `tag add <file> <tag1> [tag2...]`           | 为文件/文件夹添加标签                 | `rfe tag add src/main.rs rust code`                       |
| `tag remove <file> <tag1> [tag2...]`        | 删除文件的指定标签                   | `rfe tag remove src/main.rs old`                          |
| `tag clear <file>`                          | 删除文件的所有标签                   | `rfe tag clear src/main.rs`                               |
| `tag get <file>`                            | 查看文件的所有标签                   | `rfe tag get src/main.rs`                                 |
| `tag list`                                  | 查看所有带标签的文件                  | `rfe tag list`                                            |
| `tag find <tag-pattern1> [tag-pattern2...]` | 全局搜索匹配标签的文件，支持正则            | `rfe tag find rust code`                                  |
| `tag backup`                                | 备份标签数据                      | `rfe tag backup`                                          |
| `tag restore`                               | 从备份恢复标签数据                   | `rfe tag restore`                                         |
| `ls -tag` / `--tags`                        | 列出文件时同时显示标签                 | `rfe ls -tag`, `rfe ls -ltag`                             |
| `ls -t/--tag <tag-pattern>`                 | 列出当前目录下匹配指定标签的文件，可多次指定      | `rfe ls -t rust`, `rfe ls -lt rust`                       |
| `->`                                        | 命令链式执行，前一命令输出传递给后一命令        | `rfe pwd -> ls -> cd ..`                                  |
| `->!`                                       | 容错命令链，前一命令失败也继续执行           | `rfe cd nonexist ->! ls`                                  |
| `{}`                                        | 命令链中的占位符，插入前一命令的输出          | `rfe cppwd -> alias add desktop {}`                       |
| `exit` / `quit` / `q`                       | 退出交互式模式                     | `exit`                                                    |
| `mkdf -f <path>`                            | 创建文件，自动创建父目录                | `mkdf -f test.txt`、`mkdf -f path/to/file.txt`             |
| `mkdf -d <path>`                            | 创建文件夹                       | `mkdf -d test_folder`                                     |
| `mkdf -d -p <path>`                         | 创建文件夹和父目录                   | `mkdf -d -p parent/child/grandchild`                      |
| `mkdf -h` / `--help`                        | 查看mkdf命令帮助                  | `mkdf --help`                                             |
| `ls --re <pattern>`                         | 使用正则表达式搜索文件/目录              | `rfe ls --re \.rs$`                                       |
| `ls --re-deep <pattern>`                    | 递归使用正则表达式搜索                 | `rfe ls --re-deep \.rs$`                                  |
| `ls --re --xcaps <pattern>`                 | 大小写不敏感的正则搜索                 | `rfe ls --re --xcaps \.RS$`                               |
| `ls --re-deep --xcaps <pattern>`            | 递归大小写不敏感的正则搜索               | `rfe ls --re-deep --xcaps \.RS$`                          |
| `ESC`                                       | 在 REPL 模式下按ESC键，清空当前输入内容    | 输入命令过程中按ESC快速清空                                           |
| `-moe` / `--moe`                            | 启用 Moe Moe 萌系模式，粉色系配色+颜文字表情 | `rfe -moe`、`rfe -moe ls`、`rfe -moe pwd`                   |

### 📝 常用正则表达式语法参考

| 语法       | 功能说明            | 示例                                             |
| -------- | --------------- | ---------------------------------------------- |
| `.`      | 匹配任意单个字符        | `ls --re fi.e` → 匹配 file、fine 等                |
| `*`      | 匹配前一个字符0次或多次    | `ls --re a*` → 匹配 a、aa、aaa 等                   |
| `+`      | 匹配前一个字符1次或多次    | `ls --re a+` → 匹配 a、aa、aaa 等                   |
| `?`      | 匹配前一个字符0次或1次    | `ls --re colou?r` → 匹配 color、colour            |
| `^`      | 匹配字符串开头         | `ls --re ^src` → 匹配 src 开头的文件                  |
| `$`      | 匹配字符串结尾         | `ls --re \.rs$` → 匹配所有 .rs 文件                  |
| `[abc]`  | 匹配字符集中任意一个字符    | `ls --re [Ff]ile` → 匹配 File、file               |
| `[^abc]` | 匹配不在字符集中的任意字符   | `ls --re [^Ff]ile` → 匹配 aile、bile等             |
| `\|`     | 或逻辑，匹配左右任意一个表达式 | `ls --re \.rs$\|\.toml$` → 匹配 rs和toml文件        |
| `()`     | 分组，用于组合表达式      | `ls --re (src\|target)\/` → 匹配src或target目录下的文件 |

> 💡 正则模式默认会递归搜索当前目录及所有子目录，匹配的文件会显示为相对当前工作目录的路径，格式与普通ls命令保持一致。

### 🔖 路径别名使用说明

路径别名功能可以让你为常用的长目录设置短别名，使用时只需要在别名前加`@`即可，无需再输入冗长的路径。

- 别名持久化存储：添加的别名会自动保存到系统配置目录，重启rfe仍然可用
- 支持别名子路径：可以在别名后直接拼接子路径，比如`@docs/rust`
- 支持所有命令：别名可以在任何需要路径参数的命令中使用（ls/cd/cpf/open/mv等）
- 配置存储位置：
  - Windows: `%APPDATA%\rfe\aliases.json`
  - Linux/macOS: `~/.config/rfe/aliases.json`

### 🏷️ 标签管理使用说明

标签功能可以让你为文件和文件夹添加自定义标签，通过标签快速筛选和搜索文件，提升文件管理效率。

- 多标签支持：单个文件可以添加多个标签
- 正则匹配：标签查询支持完整的正则表达式语法
- 多标签组合查询：可以同时指定多个标签条件，查询同时匹配所有标签的文件
- 自动备份：每次修改标签都会自动备份，防止数据丢失，支持手动备份和恢复
- 配置存储位置：
  - Windows: `%APPDATA%\rfe\tags.json`、`%APPDATA%\rfe\tags.json.bak`
  - Linux/macOS: `~/.config/rfe/tags.json`、`~/.config/rfe/tags.json.bak`
- `ls -tag`参数可以在列出文件时同时显示其关联的标签，方便查看

***

## 💡 使用技巧

### 🏷️ 为文件夹设置标签的技巧

由于标签功能目前主要针对文件设计，但你可以通过一个简单的技巧为文件夹设置标签：

#### 方法：使用 `.index` 标记文件

在需要设置标签的文件夹下创建一个名为 `.index` 的隐藏文件，然后为这个 `.index` 文件设置标签。这样，当你使用 `tag find` 或 `ls -t` 命令时，可以通过标签定位到包含该 `.index` 文件的文件夹。

#### 使用步骤

1. 在目标文件夹内创建 `.index` 文件：
   ```bash
   mkdf -f /path/to/folder/.index
   ```

2. 为 `.index` 文件添加标签：
   ```bash
   tag add /path/to/folder/.index work project important
   ```

3. 搜索包含该标签的文件夹：
   ```bash
   tag find work project
   ```

4. 列出当前目录下带有指定标签的文件夹（通过 .index 文件）：
   ```bash
   ls -t work
   ```

#### 优势

- **语义清晰：`.index` 文件作为文件夹的索引标记
- **跨平台兼容：隐藏文件在 Windows/Linux/macOS 都支持
- **易于管理：可以随时修改或删除 `.index` 文件来管理文件夹的标签
- **与现有功能兼容：无需修改代码，直接使用现有的标签系统

## 🤝 贡献指南

欢迎任何形式的贡献！无论是提交 Bug 报告、功能建议还是代码贡献，都非常感谢。

### 贡献步骤

1. Fork 本仓库
2. 创建你的功能分支：`git checkout -b feature/AmazingFeature`
3. 提交你的更改：`git commit -m 'Add some AmazingFeature'`
4. 推送到分支：`git push origin feature/AmazingFeature`
5. 提交 Pull Request

### 代码规范

- 遵循 Rust 官方编码规范
- 提交代码前请运行 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查代码问题
- 确保所有测试通过：`cargo test`

## 📄 许可证

本项目采用 MIT 许可证，详情请查看 [LICENSE](LICENSE) 文件。

## 📞 联系方式

- 项目地址：[rust-file-explorer: 基于rust的CLI工具](https://gitee.com/glacier-bot/rust-file-explorer)
- 问题反馈：请提交 [Gitee Issue](https://gitee.com/glacier-bot/rust-file-explorer/issues)
- 邮箱：<1098644849@qq.com>

***

⭐ 如果这个项目对你有帮助，欢迎点个 Star 支持一下！
