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

## 🛠️ 技术栈

| 技术/依赖 | 版本 | 用途 |
|----------|------|------|
| Rust | 2021 Edition | 核心开发语言 |
| colored | 2.1 | 终端彩色输出 |
| dirs | 5.0 | 跨平台系统目录路径获取 |
| unicode-width | 0.1 | Unicode 字符串宽度计算，优化界面排版 |
| open | 5.0 | 调用系统默认应用打开文件 |
| rustyline | 12.0 | 命令行交互框架，支持补全、历史记录 |
| arboard | 3.4 | 跨平台剪贴板操作 |
| regex | 1.10 | 正则表达式搜索支持 |
| serde | 1.0 | 序列化/反序列化支持 |
| serde_json | 1.0 | JSON格式数据持久化 |

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

2. 进入项目目录：
```bash
cd rust-file-explorer
```

3. 编译发布版本：
```bash
cargo build --release
```

4. 安装二进制文件到系统路径：
- **Windows**：
  复制 `target\release\rfe.exe` 到 `C:\Windows\System32` 或其他已加入 `PATH` 的目录。

- **Linux/macOS**：
```bash
sudo cp target/release/rfe /usr/local/bin/
```

5. 验证安装：
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

| 命令 | 说明 | 示例 |
|------|------|------|
| `ls` | 列出当前目录内容 | `rfe ls` |
| `ls -l` | 列出目录详细信息（包含大小、创建时间、修改时间） | `rfe ls -l` |
| `ls -a` | 列出所有文件，包含隐藏文件 | `rfe ls -a` |
| `ls -la` | 列出所有文件的详细信息 | `rfe ls -la` |
| `ls <path>` | 列出指定目录的内容 | `rfe ls ~/Documents` |
| `ls --re <pattern>` | 使用正则表达式全局搜索文件/目录 | `rfe ls --re \.rs$`（搜索所有.rs文件） |
| `ls --re --re-insensitive <pattern>` | 大小写不敏感的正则搜索 | `rfe ls --re --re-insensitive cargo`（搜索包含cargo的文件，不区分大小写） |
| `pwd` | 打印当前工作目录路径 | `rfe pwd` |
| `cppwd` | 复制当前目录路径到剪贴板 | `rfe cppwd` |
| `cpf <file>` | 复制指定文件的绝对路径到剪贴板 | `rfe cpf README.md` |
| `cd` | 切换到用户主目录 | `rfe cd` |
| `cd ..` | 切换到上级目录 | `rfe cd ..` |
| `cd <path>` | 切换到指定目录 | `rfe cd /usr/local/bin` |
| `open <path>` | 使用系统默认应用打开文件 / 在资源管理器中打开文件夹 | `rfe open document.pdf` / `rfe open ~/Documents` |
| `mv <source> <dest>` | 移动文件/目录到目标位置 | `rfe mv file.txt ~/Documents/` |
| `mv <source> <dest> --cp` | 复制文件/目录到目标位置（保留原文件） | `rfe mv photo.jpg ~/Pictures/ --cp` |
| `clear` / `cls` | 清空终端屏幕 | `rfe clear` |
| `help` / `?` | 显示帮助信息 | `rfe help` |
| `alias add <name> <path>` | 添加路径别名 | `rfe alias add docs ~/Documents` |
| `alias remove <name>` | 删除路径别名 | `rfe alias remove docs` |
| `alias list` | 查看所有路径别名 | `rfe alias list` |
| `@<alias>` | 使用路径别名，可用于所有需要路径的命令 | `rfe ls @docs`, `rfe cd @docs/rust` |
| `tag add <file> <tag1> [tag2...]` | 为文件/文件夹添加标签 | `rfe tag add src/main.rs rust code` |
| `tag remove <file> <tag1> [tag2...]` | 删除文件的指定标签 | `rfe tag remove src/main.rs old` |
| `tag clear <file>` | 删除文件的所有标签 | `rfe tag clear src/main.rs` |
| `tag get <file>` | 查看文件的所有标签 | `rfe tag get src/main.rs` |
| `tag list` | 查看所有带标签的文件 | `rfe tag list` |
| `tag find <tag-pattern1> [tag-pattern2...]` | 全局搜索匹配标签的文件，支持正则 | `rfe tag find rust code` |
| `tag backup` | 备份标签数据 | `rfe tag backup` |
| `tag restore` | 从备份恢复标签数据 | `rfe tag restore` |
| `ls -t/--tag <tag-pattern>` | 列出当前目录下匹配指定标签的文件，可多次指定 | `rfe ls -t rust`, `rfe ls -lt rust` |
| `exit` / `quit` / `q` | 退出交互式模式 | `exit` |

### 📝 常用正则表达式语法参考
| 语法 | 功能说明 | 示例 |
|------|----------|------|
| `.` | 匹配任意单个字符 | `ls --re fi.e` → 匹配 file、fine 等 |
| `*` | 匹配前一个字符0次或多次 | `ls --re a*` → 匹配 a、aa、aaa 等 |
| `+` | 匹配前一个字符1次或多次 | `ls --re a+` → 匹配 a、aa、aaa 等 |
| `?` | 匹配前一个字符0次或1次 | `ls --re colou?r` → 匹配 color、colour |
| `^` | 匹配字符串开头 | `ls --re ^src` → 匹配 src 开头的文件 |
| `$` | 匹配字符串结尾 | `ls --re \.rs$` → 匹配所有 .rs 文件 |
| `[abc]` | 匹配字符集中任意一个字符 | `ls --re [Ff]ile` → 匹配 File、file |
| `[^abc]` | 匹配不在字符集中的任意字符 | `ls --re [^Ff]ile` → 匹配 aile、bile等 |
| `\|` | 或逻辑，匹配左右任意一个表达式 | `ls --re \.rs$\|\.toml$` → 匹配 rs和toml文件 |
| `()` | 分组，用于组合表达式 | `ls --re (src\|target)\/` → 匹配src或target目录下的文件 |

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

- 项目地址：[https://github.com/yourusername/rust-file-explorer](https://github.com/yourusername/rust-file-explorer)
- 问题反馈：请提交 [GitHub Issue](https://github.com/yourusername/rust-file-explorer/issues)
- 邮箱：your.email@example.com

---

⭐ 如果这个项目对你有帮助，欢迎点个 Star 支持一下！
