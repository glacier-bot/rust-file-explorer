# AGENTS.md - Rust File Explorer (rfe)

## Important Notes

1. 使用中文回答所有问题
2. 应用最小化改动策略，不增加无用逻辑，不修改无关代码
3. 应用最小职责原则，每个模块或函数只负责一个或一组相关的功能
4. 当单一文件代码量超过300行时，首先考虑是否存在冗余代码或逻辑复杂度过高，其次考虑是否可以拆分成多个文件，增强代码可维护性和可扩展性
5. 代码应该保证简洁，避免重复代码和复杂的逻辑，必须保证编译时没有警告或错误提示
6. 代码修改后应该保证与修改前的功能和逻辑一致，不引入新的错误或异常情况
7. 在删除文件或目录时，应该先确认用户操作，避免误删重要文件

## Commands

```bash
cargo build           # Debug build
cargo build --release # Release build (LTO, size-optimized)
cargo test            # Run unit tests
cargo fmt             # Format code (rustfmt defaults)
cargo clippy          # Static analysis (clippy defaults)
cargo run             # Launch REPL mode
cargo run -- -moe     # Launch REPL with moe (pink) theme
cargo run -- ls       # Execute single command directly
```

## Project Structure

- **Entry**: `src/main.rs` - main() handles REPL vs direct command mode
- **App layer**: `src/app/` - REPL, command pipeline/execution
- **Commands**: `src/commands/` - individual command implementations (ls, cd, mv, etc.)
- **State**: `src/managers/` - alias/tag persistence with auto-backup
- **Completion**: `src/completion/` - tab completion for commands/aliases/tags

## Critical Quirks

1. **Dual execution modes**: Same binary does interactive REPL OR direct command execution
2. **Line numbers (`-r`)**: Only work in REPL mode - requires prior `ls` to populate cache
3. **Command chaining**: `->` stops on error, `->!` continues despite errors
4. **Placeholders**: `{}` = previous output, `{}.pop` or `{}..` = up one directory
5. **Tags only work on files**: To tag dirs, create `.index` inside and tag that file
6. **Moe mode**: Global state toggle affecting all colored output
7. **ESC in REPL**: Clears entire input line (custom rustyline binding)

## Storage

- Config: `%APPDATA%\rfe\` (Windows) or `~/.config/rfe/` (Unix)
- Auto `.bak` files created on each alias/tag modification

## Dependencies

- Rust 1.65.0+, Edition 2021
- rustyline for REPL, colored for output, crossterm for terminal events
- Release profile: lto=true, codegen-units=1, panic=abort, opt-level="z"
