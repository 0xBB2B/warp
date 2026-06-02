# Git Graph Panel — Technical Spec (as built)

## Context

A read-only commit DAG visualization tab (`ToolPanelView::GitGraph`) in the left
tools panel. It follows the git repository of the currently active pane, renders
the commit graph, shows a commit's detail on click, opens a read-only file diff
in the main area, supports a repository picker, branch filtering, manual refresh,
and "load more" pagination. No write operations. Gated by `FeatureFlag::GitGraph`
plus the `show_git_graph` user setting.

### Key technical constraints
1. **The render layer only has rectangle-family primitives.** `Scene` in
   `crates/warpui_core/src/scene.rs` provides only `Rect` (with `Border` /
   `CornerRadius` / `DropShadow` / `Dash`), `Image`, `Glyph`, `Icon` — **no
   line / path / bezier, no rotation.** DAG connectors are therefore drawn as
   **orthogonal polylines** (thin vertical/horizontal rects) with square corners;
   rounded bends are deferred. This is an explicit trade-off vs git-graph's beziers.
2. **Custom drawing goes through `Element::paint`.** `row_canvas.rs` implements a
   custom `Element` whose `paint` calls `ctx.scene.draw_rect_with_hit_recording`
   (pattern mirrors `crates/warpui_core/src/elements/rect.rs`). A node dot is a
   small square with `corner_radius = half side` (→ circle).
3. **No git library.** Data is fetched by shelling out to `git` (async).
4. **`metal` toolchain required to build on macOS** (`warpui/build.rs`
   unconditionally compiles Metal shaders).

## Implementation

### Module structure
```
app/src/workspace/view/git_graph/
  mod.rs          declares submodules; re-exports GitGraphView
  data.rs         data types + git log/show/branch/diff parsing (pure) + async fetch
  layout.rs       pure lane-layout algorithm (assign_lanes)
  row_canvas.rs   GitGraphRowCanvas: custom Element painting one row's lanes
  view.rs         GitGraphView + GitGraphAction + GitGraphEvent
  data_tests.rs / layout_tests.rs   unit tests
app/src/code/commit_diff_view.rs           read-only commit-file diff view
app/src/pane_group/pane/commit_diff_pane.rs host pane for the diff view
app/src/settings/git.rs                    GitSettings (show_git_graph, scan depth)
app/src/settings_view/git_page.rs          "Git" settings page UI
```
State is held directly in `GitGraphView` (single, unshared view); no separate
model module — a `GitGraphModel` would be premature.

### Data layer (data.rs)
```
struct CommitNode  { hash, short_hash, parents: Vec<String>, author_name,
                     author_email, author_time: i64, subject, refs: Vec<RefLabel> }
enum   RefKind     { Head, LocalBranch, RemoteBranch, Tag }
struct RefLabel    { kind: RefKind, name: String }
struct BranchRef   { full ref + display name + kind }       // branch-filter list
struct ChangedFile { path: String, additions: u32, deletions: u32 }
struct CommitDetail{ committer_name, committer_time: i64, message, files }
struct CommitFileDiff { base_content: String, hunks: Vec<DiffHunk> } // file diff
```
Async fetch + pure parsers (unit-tested; `load_*` wrappers are thin):
- `discover_repositories(anchor, depth)` — scans the anchor dir and (down to
  `depth` levels) finds git repo roots for the repository picker.
- `load_branches(dir)` — local + remote branch refs for the branch filter.
- `load_commit_graph(dir, filter, limit, skip)` — `git log --all --date-order
  --decorate=full` with `%x1f`/`%x1e` separators; `filter` restricts to the
  selected branches; `limit`/`skip` drive pagination.
- `load_commit_detail(dir, hash)` — `git show --numstat` → committer + message +
  changed files.
- `load_file_diff_at_commit(dir, hash, path)` — the file's parent-commit content
  + unified `DiffHunk`s for the read-only diff pane (compares against first
  parent; root commit falls back to whole-file additions).

### Lane layout (layout.rs) — core algorithm
Input `&[CommitNode]` (newest→oldest). Output `GraphLayout { rows, max_lanes }`
where each `GraphRow` carries `node_col`, `node_color`, `node_continues_up`,
`passing`, `to_parents`, `from_children`. Top-down scan maintaining
`lanes: Vec<Option<Lane>>`; first parent continues the node's column (a merged
branch visually rejoins the mainline), extra parents open new lanes. **No lane
compaction** — a lane keeps its column for life, so adjacent rows align and each
row paints independently. Covered for linear / fork / merge / octopus /
multi-root / freed-lane-reuse shapes.

### Per-row painting (row_canvas.rs)
`GitGraphRowCanvas { row, lane_count }` implements `Element`: width
`lane_count * LANE_WIDTH`, fixed `ROW_HEIGHT`. `paint` draws 2px rects —
`passing` verticals, `node_continues_up` top→mid, `from_children` (vertical +
horizontal elbow), `to_parents` (horizontal + vertical elbow), and the commit
dot. Colors from a fixed 7-entry `PALETTE`.

### View (view.rs)
`GitGraphView` holds all state: `scan_anchor`, `repositories` + `selected_repo` +
`repo_dropdown`, `branches` + `selected_branches` + `saved_branch_selections` +
branch-filter overlay state, `commits` + `layout` + `state`
(NoRepo/Loading/Loaded/Error), `selected` + `detail`, list/detail scroll states,
a draggable detail-area height (`ResizableState`), and `has_more`/`loading_more`.
`GitGraphAction` = `SelectCommit` | `SelectRepository` | `Refresh` | `LoadMore` |
branch-filter toggles | `OpenFileDiff(idx)`. Layout: header (commit count +
repository picker when >1 repo + branch-filter button + refresh) over a single
column with `Shrinkable` factors (graph list alone, or list + detail when a
commit is selected). The detail area (message + author/committer + full hash +
changed files) is one `ClippedScrollable`; its height is user-draggable.

**Active-repo resolution**: `LeftPanelView`'s
`WorkingDirectoriesEvent::DirectoriesChanged` handler pushes the most-recent
local directory into `GitGraphView::set_working_directory`, which re-runs
`discover_repositories`. `git log` resolves the repo from any subdirectory.

### Read-only file diff pane
Clicking a changed file dispatches `GitGraphAction::OpenFileDiff(idx)`; the view
emits `GitGraphEvent::OpenCommitFileDiff { repo_relative_path, short_hash,
base_content, hunks }`, forwarded up by the left panel to the workspace, which
builds a `CommitDiffView` (`app/src/code/commit_diff_view.rs`) hosted in a
`CommitDiffPane`. The diff renders through the **existing code-review diff
machinery** — `hunks` are converted via
`code_review::diff_state::convert_hunks_to_diff_deltas` and shown in a
`CodeEditorView`, so commit-file diffs reuse the same editor/diff rendering.
Re-clicking another file reuses the first visible commit-diff pane (updates its
content in place) instead of opening a new one. The pane is non-restorable
(`source: None`) so a historical revision is never written back to the working
tree.

### Settings (`Settings → Git`)
`GitSettings` (`app/src/settings/git.rs`, via `define_settings_group!`):
- `show_git_graph: bool` (default true; toml `git.show_graph_panel`) — gates the
  toolbelt tab when `FeatureFlag::GitGraph` is on.
- `git_graph_scan_depth: u32` (default 1; toml `git.graph_scan_depth`) — how many
  directory levels below the working directory `discover_repositories` probes.
The "Git" settings page (`app/src/settings_view/git_page.rs`) surfaces both.

### Integration points (wiring)
- `left_panel.rs`: `ToolPanelView::GitGraph` + `LeftPanelAction::GitGraph`,
  `git_graph_view` field, toolbelt button (`Icon::GitBranch`), working-directory
  subscription, and `LeftPanelEvent::OpenCommitFileDiff` forwarding.
- `workspace/view.rs`: `compute_left_panel_views` adds `GitGraph` when
  `cfg!(feature="local_fs") && FeatureFlag::GitGraph.is_enabled()`; builds the
  `CommitDiffView` on the forwarded event.
- `app_state.rs`: `LeftPanelDisplayedTab::GitGraph` snapshot mapping.
- Feature flag: cargo feature `git_graph` (`app/Cargo.toml`, not default);
  `FeatureFlag::GitGraph` (`crates/warp_features/src/lib.rs` + `DOGFOOD_FLAGS`);
  compile→runtime bridge in `app/src/features.rs`.
- `crates/warpui_core/src/elements/resizable.rs`: `dragbar_hover_color` support
  used by the draggable detail-area splitter.

## Testing and validation
- **Unit tests** (`data_tests.rs` / `layout_tests.rs`): `assign_lanes` across DAG
  shapes (invariants 4–5); `parse_commit_log` / `parse_decorate` /
  `parse_commit_detail` / `parse_numstat` / repo discovery edge cases
  (invariants 6, 8, 10).
- **Integration test** (`crates/integration/src/test/git_graph.rs`,
  `test_git_graph_loads_commits`): builds a real repo, opens the panel, asserts
  the graph loads with commits — end-to-end coverage of invariants 1–4
  (entry point → working-dir follow → render). Drives the panel via `pub(crate)`
  test accessors on `GitGraphView` / `LeftPanelView` / `WorkspaceView` and a
  helper in `app/src/integration_testing/git_graph.rs`.
- Manual verification under `--features local_fs,git_graph`.

## Non-goals (deferred)
- Write operations (checkout / branch / merge / rebase / cherry-pick / revert /
  reset / stash / tag / push / pull).
- Auto-refresh on repo change — needs repo-watcher plumbing + debounce; manual
  refresh covers it.
- Rounded (bezier) connectors — render-layer limitation; orthogonal corners today.
- In-graph commit search.
- Per-file A/M/D/R status and formatted commit timestamps in the detail area.
- Theme-token colors — fixed palette today.
