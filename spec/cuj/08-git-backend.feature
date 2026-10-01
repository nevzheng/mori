Feature: CUJ 8 - trees as git worktrees
  `mori clone --vcs git` makes a plain git clone, and every tree mori creates in it is a detached
  git worktree, where git and the tools built on it work. The safety rules are the same as for jj.
  jj stays the default.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone --vcs git github.com/acme/widget"

  Scenario: A git clone is recorded like any other
    When I run "mori ls --json"
    Then it succeeds
    And the clone is a git repo without jj
    And mori records "github.com/acme/widget" with a pinned base tree
    And the forest shows "default" in "github.com/acme/widget" as a clean base tree, pinned

  Scenario: A git tree is a detached worktree where git works
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    When I run "mori ls --json"
    Then it succeeds
    And the clone has a git worktree "claude-fix-login"
    And git works in "claude-fix-login" on a detached HEAD
    And the forest shows "claude-fix-login" as claude's task "fix-login", lifetime "task-done"

  Scenario: tree create says a git tree is a git worktree
    When I run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    Then it succeeds
    And the output says the tree is a git worktree on a detached HEAD

  Scenario: A git tree with a commit on no remote branch is kept
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And someone commits work with git in "claude-fix-login"
    When I run "mori tree remove github.com/acme/widget claude-fix-login"
    Then it fails with status FAILED_PRECONDITION and reason "TREE_HAS_UNSAVED_WORK"
    And "<home>/mori/trees/widget/claude-fix-login/login.rs" exists

  Scenario: A clean git tree is removed
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    When I run "mori tree remove github.com/acme/widget claude-fix-login"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"
    And the clone has no git worktree "claude-fix-login"
    And mori has no record of the tree "claude-fix-login"

  Scenario: gc removes a landed git tree, and restore brings it back
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And someone commits work with git in "claude-fix-login"
    And pushes it with git to the remote as "claude/fix-login"
    And I have run "mori ls"
    And the remote deletes the git branch "claude/fix-login" after a squash merge
    And I have removed it with "mori gc --apply --yes"
    When I restore the removed tree
    Then it succeeds
    And "<home>/mori/trees/widget/claude-fix-login/login.rs" exists
    And the clone has a git worktree "claude-fix-login"
    And the journal has an entry for "claude-fix-login" whose commit is pinned
