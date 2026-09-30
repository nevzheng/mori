Feature: CUJ 7 - undo a removal
  `mori restore` brings back a tree `mori gc apply` removed: its workspace on the pinned commit, at
  its old path, under its old record. If the commit is gone, it says so and changes nothing.

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
    And I have applied the report

  Scenario: A removed tree comes back, after its bookmark is gone
    When I restore the removed tree
    Then it succeeds
    And "<home>/mori/trees/widget/claude-fix-login/login.rs" exists
    And the clone has a workspace "claude-fix-login"
    And mori records the tree "claude-fix-login" under its old ID

  Scenario: A tree can only come back once
    Given I have restored the removed tree
    When I restore the removed tree
    Then it fails with status ALREADY_EXISTS and reason "TREE_EXISTS"

  Scenario: Restore says so when the commit is gone
    Given the removed tree's commit is gone from the clone
    When I restore the removed tree
    Then it fails with status FAILED_PRECONDITION and reason "RESTORE_COMMIT_GONE"
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"

  Scenario: An unknown entry is not found
    When I run "mori restore j-0-00000000"
    Then it fails with status NOT_FOUND and reason "ENTRY_NOT_FOUND"
