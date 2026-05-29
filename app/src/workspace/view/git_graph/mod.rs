//! Git Graph 面板：只读的 commit DAG 可视化（见 specs/git-graph）。
//!
//! 模块分层：
//! - [`data`]   提交数据类型 + `git log` 输出解析（纯函数）+ 异步取数。
//! - [`layout`] 把提交序列编排成逐行的泳道布局（纯函数，核心算法）。
//! - [`view`]   左侧 panel 的 Git Graph 视图（Phase 2 为纯文本列表）。
//!
//! 后续阶段会补充图谱泳道绘制（接入 [`layout`]）与提交详情。

pub(crate) mod data;
pub(crate) mod layout;
pub(crate) mod view;

pub(crate) use view::GitGraphView;
