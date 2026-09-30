Feature: CUJ 2 - see the forest
  `mori ls` shows every repo mori manages and every tree in it: who it's for, what task, how long
  it may live, and whether it holds work that exists only on this machine. It reads jj without
  snapshotting, so it never disturbs someone working in a tree. Mismatches between mori's records
  and jj are rows, never errors.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone github.com/acme/widget"

  Scenario: A fresh clone shows its base tree
    When I run "mori ls --json"
    Then it succeeds
    And the forest shows "default" in "github.com/acme/widget" as a clean base tree, pinned

  Scenario: A task tree shows its owner, task and lifetime
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    When I run "mori ls --json"
    Then the forest shows "claude-fix-login" as claude's task "fix-login", lifetime "task-done"

  Scenario: Work in a tree shows as edited and unpushed
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And someone edits "claude-fix-login" and runs jj there
    When I run "mori ls --json"
    Then the forest shows "claude-fix-login" as edited with 1 unpushed change

  Scenario: A workspace mori didn't make is foreign
    Given someone added a workspace "scratch" to the clone outside mori
    When I run "mori ls --json"
    Then it succeeds
    And the forest shows "scratch" as foreign

  Scenario: A tree whose workspace is gone is missing
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And someone forgot the workspace "claude-fix-login" outside mori
    When I run "mori ls --json"
    Then it succeeds
    And the forest shows "claude-fix-login" as missing

  Scenario: Clones mori didn't make are listed apart
    Given "<home>/mori/repos/github.com/acme/other/.git" exists
    When I run "mori ls --json"
    Then it succeeds
    And the forest lists "github.com/acme/other" as not managed by mori

  Scenario: One repo at a time
    When I run "mori ls git@github.com:acme/widget.git --json"
    Then it succeeds
    And the forest shows only "github.com/acme/widget"

  Scenario: A repo mori doesn't manage is refused
    When I run "mori ls github.com/acme/gadget"
    Then it fails with status NOT_FOUND and reason "REPO_NOT_MANAGED"

  Scenario: The text view is a table per repo
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    When I run "mori ls"
    Then it succeeds
    And the output has a row for "claude-fix-login" with "claude", "fix-login" and "task-done"

  Scenario: Looking changes nothing
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    When I run "mori ls"
    Then no file or directory was created or modified
