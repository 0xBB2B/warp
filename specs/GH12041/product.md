# Git Graph Panel (read-only commit DAG visualization)

## Summary
A read-only "Git Graph" tab in the left tools panel that renders the active
repository's commit history as a colored directed acyclic graph (DAG) — branch
lanes, commit nodes, merge/fork connectors, and HEAD / local-branch /
remote-branch / tag badges. Selecting a commit shows its details and changed
files; clicking a changed file opens a read-only diff in the main area. Inspired
by [vscode-git-graph](https://github.com/mhutchie/vscode-git-graph). Strictly
read-only — no operation mutates repository state.

## Problem
Warp is a terminal, so users can run `git log --graph`, but:
- ASCII graphs become unreadable with many branches and are not interactive — you
  can't click a commit to inspect what it changed.
- The tools panel (Project Explorer / Global Search / Warp Drive / Agent
  Conversations) has no visual git entry point, so "looking at git history" — a
  high-frequency task — forces a switch to the terminal or an external tool.

## Behavior invariants

### Entry point & visibility
1. When `FeatureFlag::GitGraph` is enabled, the build includes the `local_fs`
   feature, and the `Settings → Git` toggle `show_git_graph` is on, a "Git Graph"
   button (`Icon::GitBranch`, tooltip "Git Graph") appears in the left toolbelt.
2. Clicking the button opens the Git Graph view; clicking another tab switches
   away, consistent with the other left-panel tabs.
3. The panel follows the repository of the currently active pane's working
   directory; changing the active directory re-resolves the repository.

### Graph rendering
4. Each commit is one fixed-height row: a lane graph (colored vertical lines +
   node dot + orthogonal connectors) followed by short hash, ref badges, subject.
5. Merge commits, forks, and multi-root repositories all connect correctly; lane
   width adapts to the current maximum number of parallel branches.
6. Ref badges distinguish four kinds — HEAD, local branch, remote branch, tag —
   each as a color-coded badge.

### Commit detail & file diff
7. With no commit selected, the detail area is empty.
8. Selecting a commit shows: full message, author (name + email), committer (when
   it differs from the author), full hash, and the changed-file list (each row:
   path + `+insertions / -deletions`).
9. Clicking a changed file opens a read-only diff of that file's changes in the
   selected commit, in a pane in the main area; clicking another file reuses the
   same diff pane rather than opening a new one.

### Multi-repository & branch filtering
10. When the working directory hosts multiple repositories (within
    `git_graph_scan_depth`), a repository picker at the top lets the user switch
    which repository's history is shown.
11. A branch-filter overlay lets the user choose which branches' commits the graph
    shows (select-all / deselect-all / per-branch toggles); the graph updates to
    match the selection.

### Pagination & refresh
12. The graph loads an initial page (~200 commits); a "Load more" row at the
    bottom fetches the next page on demand.
13. A manual refresh button in the header re-fetches the current repository.

### Settings
14. `Settings → Git` exposes `show_git_graph` (panel visibility toggle) and
    `git_graph_scan_depth` (how many directory levels below the working directory
    to probe for repositories: 0 = the working directory's own repo only).

### States & edge cases
15. Active pane is not inside any git repository: show "Current directory is not a
    git repository."
16. A `git` command fails: show an error message; the header refresh button lets
    the user retry. Never crash and never affect other panels.
17. Repository has no commits: show "No commits yet."
18. Strictly read-only: no checkout / branch / merge / rebase / etc. — the panel
    never mutates repository state under any interaction.

## Non-goals (deferred)
- **Write operations**: checkout / create branch / merge / rebase / cherry-pick /
  revert / reset / stash / tag / push / pull / context-menu actions.
- **Auto-refresh** on repository changes (new commit, branch switch): manual
  refresh covers this; auto-refresh needs repo-watcher plumbing + debounce.
- **Rounded (bezier) connectors**: the render layer has only rectangle
  primitives, so connectors are orthogonal square-corner polylines.
- **In-graph commit search.**
- **Per-file A/M/D/R status** and **formatted commit timestamps** in the detail
  area (only adds/dels counts and raw metadata are shown today).
- **Theme-token colors**: lane and badge colors come from a fixed palette today.
