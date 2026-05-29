//! Git Graph 面板：只读的 commit DAG 可视化（见 specs/git-graph）。
//!
//! 模块分层（当前为 Phase 1，仅数据层 + 布局算法）：
//! - [`data`]   提交数据类型 + `git log` 输出解析（纯函数）+ 异步取数。
//! - [`layout`] 把提交序列编排成逐行的泳道布局（纯函数，核心算法）。
//!
//! 后续阶段会补充 model / view / row_canvas（状态机与渲染）。

pub(crate) mod data;
pub(crate) mod layout;
