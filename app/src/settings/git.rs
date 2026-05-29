use settings::macros::define_settings_group;
use settings::{RespectUserSyncSetting, SupportedPlatforms, SyncToCloud};

// Git 相关设置。目前仅控制 Git Graph 面板的显隐；后续 git 相关参数（如默认分支过滤、
// 连线样式等）也归到这一组，对应设置里独立的 "Git" 分页。
define_settings_group!(GitSettings, settings: [
    // 用户偏好：在 FeatureFlag::GitGraph 对当前渠道开启的前提下，控制 Git Graph 面板是否
    // 出现在左侧 tools panel（与 show_project_explorer / show_global_search 同类的 tab 显隐开关）。
    show_git_graph: ShowGitGraph {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "git.show_graph_panel",
        description: "Whether the Git Graph panel is shown in the tools panel.",
    },
]);
