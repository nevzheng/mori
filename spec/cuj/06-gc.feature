Feature: CUJ 6 - clean up safely: the report
  `mori gc` sorts every tree into remove, blocked, keep or never, with the reason and the facts.
  Without --apply it changes no tree.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"

  Scenario: The report sorts every tree and changes none
    Given someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    And I have run "mori ls"
    And the remote deletes "claude/fix-login" after a squash merge
    And I have run "mori tree create github.com/acme/widget --agent claude --task wip"
    And I have run "mori tree create github.com/acme/widget --agent claude --task lead --lifetime pinned"
    When I run "mori gc --json"
    Then it succeeds
    And the report has "claude-fix-login" as "remove" because "LANDED"
    And the report has "claude-wip" as "keep" because "NOT_YET"
    And the report has "claude-lead" as "never" because "PINNED"
    And the report has "default" as "never" because "BASE"
    And "<home>/mori/trees/widget/claude-fix-login" exists

  Scenario: Work newer than what landed is blocked
    Given someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    And I have run "mori ls"
    And the remote deletes "claude/fix-login" after a squash merge
    And someone commits more work in "claude-fix-login"
    When I run "mori gc --json"
    Then the report has "claude-fix-login" as "blocked" because "UNSAVED"

  Scenario: A stack pushed as one bookmark lands every tree in it
    Given someone commits work in "claude-fix-login"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-logout"
    And someone stacks work in "claude-fix-logout" on top of "claude-fix-login"
    And pushes the top of "claude-fix-logout" to the remote as "claude/fix-logout"
    And I have run "mori ls"
    And the remote deletes "claude/fix-logout" after a squash merge
    When I run "mori gc --json"
    Then the report has "claude-fix-logout" as "remove" because "LANDED"
    And the report has "claude-fix-login" as "remove" because "LANDED"

  Scenario: An open bookmark with no answer from GitHub isn't a candidate
    Given someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    When I run "mori gc --json"
    Then the report has "claude-fix-login" as "keep" because "UNKNOWN"

  Scenario: A workspace mori didn't make is never removed
    Given someone added a workspace "scratch" to the clone outside mori
    When I run "mori gc --json"
    Then the report has "scratch" as "never" because "FOREIGN"

  Scenario: The text report says what to run next
    Given someone commits work in "claude-fix-login"
    And pushes it to the remote as "claude/fix-login"
    And I have run "mori ls"
    And the remote deletes "claude/fix-login" after a squash merge
    When I run "mori gc"
    Then it succeeds
    And the output says how to apply the report

  Scenario: A repo mori doesn't manage is refused
    When I run "mori gc github.com/acme/gadget"
    Then it fails with status NOT_FOUND and reason "REPO_NOT_MANAGED"
