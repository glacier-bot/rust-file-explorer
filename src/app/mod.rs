
//! 应用层模块
//! 负责 REPL 交互、CLI 参数解析和命令管道执行等高层应用逻辑

pub mod cli;
mod exec;
pub mod pipeline;
pub mod repl;
