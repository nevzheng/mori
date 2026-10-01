Feature: CUJ 6 - clean up safely: removing what the report finds
  `mori gc --apply --yes` removes a batch of the trees the report finds removable. It checks each
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

  Scenario: A confirmed batch is removed, pinned and journalled
    When I run "mori gc --apply --yes"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"
    And the clone has no workspace "claude-fix-login"
    And mori has no record of the tree "claude-fix-login"
    And the journal has an entry for "claude-fix-login" whose commit is pinned

  Scenario: Nothing is removed without confirmation
    When I run "mori gc --apply"
    Then it fails with status FAILED_PRECONDITION and reason "CONFIRMATION_NEEDED"
    And "<home>/mori/trees/widget/claude-fix-login" exists

  Scenario: A dry run removes nothing
    When I run "mori gc --apply --dry-run"
    Then it succeeds
    And the output says "claude-fix-login" would be removed
    And "<home>/mori/trees/widget/claude-fix-login" exists

  Scenario: A tree edited since the report is kept
    Given someone edits "claude-fix-login" without running jj
    When I run "mori gc --apply --yes"
    Then it succeeds
    And the output says "claude-fix-login" was kept because it has unsaved work
    And "<home>/mori/trees/widget/claude-fix-login/login.rs" exists

  Scenario: A tree whose working copy another workspace rebased is still removed
    Given trunk moves and another workspace rebases the working copy of "claude-fix-login" onto it
    When I run "mori gc --apply --yes"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"
    And the journal has an entry for "claude-fix-login" whose commit is pinned
