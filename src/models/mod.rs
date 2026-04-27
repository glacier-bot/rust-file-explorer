//! 数据模型模块
//! 包含文件信息等数据结构

use colored::Color;
use std::time::SystemTime;

/// 文件信息结构体
/// 存储文件的各种属性
pub struct FileInfo {
    /// 文件名
    pub name: String,
    /// 文件图标
    pub icon: &'static str,
    /// 文件颜色
    pub color: Color,
    /// 文件大小（字节）
    pub size: u64,
    /// 创建时间
    pub created: Option<SystemTime>,
    /// 修改时间
    pub modified: SystemTime,
    /// 是否为目录
    pub is_dir: bool,
    /// 文件标签
    pub tags: Vec<String>,
}