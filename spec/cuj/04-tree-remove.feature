Feature: CUJ 4 - an agent finishes and removes its own tree
  `mori tree remove` removes a task tree mori made, only when nothing in it exists only on this
  machine: no edits and no change missing from the remote ("pushed = safe"). It snapshots the
  tree first, so recent edits count. It never removes the clone itself or a workspace mori
  didn't make.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"

  Scenario: A clean tree is removed
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent claude"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"
    And the clone has no workspace "claude-fix-login"
    And mori has no record of the tree "claude-fix-login"

  Scenario: Edits jj hasn't seen yet still block removal
    Given someone edits "claude-fix-login" without running jj
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent claude"
    Then it fails with status FAILED_PRECONDITION and reason "TREE_HAS_UNSAVED_WORK"
    And "<home>/mori/trees/widget/claude-fix-login/login.rs" exists

  Scenario: Committed but unpushed work blocks removal
    Given someone commits work in "claude-fix-login"
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent claude"
    Then it fails with status FAILED_PRECONDITION and reason "TREE_HAS_UNSAVED_WORK"

  Scenario: Pushed work is safe, so the tree may go
    Given someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent claude"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"

  Scenario: A squash-merged tree is safe to remove
    Given someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    And I have run "mori ls"
    And the remote deletes "claude/fix-login" after a squash merge
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent claude"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"

  Scenario: Work newer than what landed still blocks removal
    Given someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    And I have run "mori ls"
    And the remote deletes "claude/fix-login" after a squash merge
    And someone commits more work in "claude-fix-login"
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent claude"
    Then it fails with status FAILED_PRECONDITION and reason "TREE_HAS_UNSAVED_WORK"

  Scenario: Only the owner removes a tree
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent codex"
    Then it fails with status FAILED_PRECONDITION and reason "NOT_TREE_OWNER"
    And "<home>/mori/trees/widget/claude-fix-login" exists

  Scenario: The clone itself is never removed
    When I run "mori tree remove github.com/acme/widget default --agent you --pinned"
    Then it fails with status FAILED_PRECONDITION and reason "BASE_TREE"

  Scenario: A workspace mori didn't make is left alone
    Given someone added a workspace "scratch" to the clone outside mori
    When I run "mori tree remove github.com/acme/widget scratch --agent claude"
    Then it fails with status ALREADY_EXISTS and reason "WORKSPACE_EXISTS"
    And the clone has a workspace "scratch"

  Scenario: A pinned tree goes only when asked for
    Given I have run "mori tree create github.com/acme/widget --agent claude --task lead --lifetime pinned"
    When I run "mori tree remove github.com/acme/widget claude-lead --agent claude"
    Then it fails with status FAILED_PRECONDITION and reason "TREE_PINNED"
    When I run "mori tree remove github.com/acme/widget claude-lead --agent claude --pinned"
    Then it succeeds

  Scenario: A missing tree only loses its record
    Given someone forgot the workspace "claude-fix-login" outside mori
    When I run "mori tree remove github.com/acme/widget claude-fix-login --agent claude"
    Then it succeeds
    And mori has no record of the tree "claude-fix-login"
    And "<home>/mori/trees/widget/claude-fix-login" exists

  Scenario: Dry run removes nothing
    When I run "mori tree remove --dry-run github.com/acme/widget claude-fix-login --agent claude"
    Then it succeeds
    And "<home>/mori/trees/widget/claude-fix-login" exists
    And the clone has a workspace "claude-fix-login"

  Scenario: An unknown tree is not found
    When I run "mori tree remove github.com/acme/widget nobody-nothing --agent claude"
    Then it fails with status NOT_FOUND and reason "TREE_NOT_FOUND"
