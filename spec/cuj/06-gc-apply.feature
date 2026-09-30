Feature: CUJ 6 - clean up safely: applying a report
  `mori gc apply` removes a confirmed batch of the trees a report lists to remove. It checks each
  tree again first and keeps any that changed; each removal pins the tree's commit and goes into
  the journal.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    And I have run "mori ls"
    And the remote deletes "claude/fix-login" after a squash merge
    And I have made a cleanup report

  Scenario: A confirmed batch is removed, pinned and journalled
    When I apply the report with "--yes"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"
    And the clone has no workspace "claude-fix-login"
    And mori has no record of the tree "claude-fix-login"
    And the journal has an entry for "claude-fix-login" whose commit is pinned

  Scenario: Nothing is removed without confirmation
    When I apply the report without confirming
    Then it fails with status FAILED_PRECONDITION and reason "CONFIRMATION_NEEDED"
    And "<home>/mori/trees/widget/claude-fix-login" exists

  Scenario: A dry run removes nothing
    When I apply the report with "--dry-run"
    Then it succeeds
    And the output says "claude-fix-login" would be removed
    And "<home>/mori/trees/widget/claude-fix-login" exists

  Scenario: A tree edited since the report is kept
    Given someone edits "claude-fix-login" without running jj
    When I apply the report with "--yes"
    Then it succeeds
    And the output says "claude-fix-login" was kept because it has unsaved work
    And "<home>/mori/trees/widget/claude-fix-login/login.rs" exists

  Scenario: An unknown report is refused
    When I run "mori gc apply gc-0-00000000 --yes"
    Then it fails with status NOT_FOUND and reason "REPORT_NOT_FOUND"
