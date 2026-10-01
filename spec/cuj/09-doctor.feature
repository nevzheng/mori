Feature: CUJ 9 - check the root
  `mori doctor` checks the root, every clone and every tree against the VCS and the disk, and lists
  each problem with the command that fixes it. It reads only, and findings don't change its exit
  code: it fails only when it can't check.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"

  Scenario: A healthy root has no problems
    Given I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    When I run "mori doctor"
    Then it succeeds
    And stdout matches "no problems"

  Scenario: A tree deleted by hand is a problem with a fix
    Given I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And the directory of "claude-fix-login" was deleted by hand
    When I run "mori doctor --json"
    Then it succeeds
    And doctor reports "TREE_DIR_GONE" for "github.com/acme/widget claude-fix-login", fixable

  Scenario: A workspace mori didn't make is reported, not a problem
    Given I have run "mori clone github.com/acme/widget"
    And someone added a workspace "theirs" to the clone outside mori
    When I run "mori doctor --json"
    Then it succeeds
    And doctor reports "FOREIGN_WORKSPACE" for "github.com/acme/widget theirs", not fixable

  Scenario: A missing context folder is a warning
    Given I have run "mori clone github.com/acme/widget"
    And the directory "<home>/mori/context/projects/widget" was deleted by hand
    When I run "mori doctor --json"
    Then it succeeds
    And doctor reports "CONTEXT_FOLDER_MISSING" for "github.com/acme/widget", fixable

  Scenario: A leftover directory under trees/ is reported
    Given I have run "mori clone github.com/acme/widget"
    And "<home>/mori/trees/widget/old-work/notes.txt" exists
    When I run "mori doctor --json"
    Then it succeeds
    And doctor reports "DIR_WITHOUT_WORKSPACE" for "<home>/mori/trees/widget/old-work", not fixable

  Scenario: A conflicted bookmark is a problem with the commands to resolve it
    Given I have run "mori clone github.com/acme/widget"
    And the bookmark "main" in the clone has two targets
    When I run "mori doctor --json"
    Then it succeeds
    And doctor reports "CONFLICTED_BOOKMARK" for "github.com/acme/widget", not fixable

  Scenario: A git clone that someone ran jj git init in is a problem
    Given I have run "mori clone --vcs git github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And someone runs "jj git init" in the clone
    When I run "mori doctor --json"
    Then it succeeds
    And doctor reports "BACKEND_CHANGED" for "github.com/acme/widget", not fixable

  Scenario: Doctor changes nothing
    Given I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And the directory of "claude-fix-login" was deleted by hand
    When I run "mori doctor"
    Then mori records the tree "claude-fix-login" for claude's task "fix-login", lifetime "task-done"

  Scenario: --fix forgets a tree deleted by hand, and restore brings it back
    Given I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And someone commits work in "claude-fix-login"
    And the directory of "claude-fix-login" was deleted by hand
    And I have fixed it with "mori doctor --fix --yes"
    When I restore the removed tree
    Then it succeeds
    And "<home>/mori/trees/widget/claude-fix-login/login.rs" exists
    And the clone has a workspace "claude-fix-login"

  Scenario: --fix needs a confirmation
    Given I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And the directory of "claude-fix-login" was deleted by hand
    When I run "mori doctor --fix"
    Then it fails with status FAILED_PRECONDITION and reason "CONFIRMATION_NEEDED"
    And mori records the tree "claude-fix-login" for claude's task "fix-login", lifetime "task-done"

  Scenario: --fix --dry-run repairs nothing
    Given I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And the directory of "claude-fix-login" was deleted by hand
    When I run "mori doctor --fix --dry-run"
    Then stdout matches "(?s)Would fix TREE_DIR_GONE  github.com/acme/widget claude-fix-login.*"
    And mori records the tree "claude-fix-login" for claude's task "fix-login", lifetime "task-done"

  Scenario: --fix repairs a deleted git worktree and a missing context folder
    Given I have run "mori clone --vcs git github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    And the directory of "claude-fix-login" was deleted by hand
    And the directory "<home>/mori/context/projects/widget" was deleted by hand
    When I run "mori doctor --fix --yes"
    Then it succeeds
    And the clone has no git worktree "claude-fix-login"
    And mori has no record of the tree "claude-fix-login"
    And "<home>/mori/context/projects/widget/README.md" exists

