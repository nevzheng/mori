Feature: CUJ 3 - an agent starts a task in its own tree
  `mori tree create` gives one piece of work its own jj workspace under trees/<repo>/, named from
  the owner and the task, on a new change on top of trunk, and records it.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone github.com/acme/widget"

  Scenario: An agent gets its own tree
    When I run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    Then it succeeds
    And "<home>/mori/trees/widget/claude-fix-login/README.md" exists
    And the clone has a workspace "claude-fix-login"
    And mori records the tree "claude-fix-login" for claude's task "fix-login", lifetime "task-done"
    And the output says it created "claude-fix-login"
    And the output says the tree is a jj workspace and points to the "vcs-in-mori" skill

  Scenario: The owner defaults to the login name
    Given USER is "Tester"
    When I run "mori tree create github.com/acme/widget --task fix-login"
    Then it succeeds
    And "<home>/mori/trees/widget/tester-fix-login" exists

  Scenario: An agent's harness can name it once, in MORI_AGENT
    Given MORI_AGENT is "codex"
    When I run "mori tree create github.com/acme/widget --task fix-login"
    Then it succeeds
    And "<home>/mori/trees/widget/codex-fix-login" exists

  Scenario: A long-lived coordinating tree is a pinned task tree
    When I run "mori tree create github.com/acme/widget --agent claude --task lead --lifetime pinned"
    Then it succeeds
    And mori records the tree "claude-lead" for claude's task "lead", lifetime "pinned"

  Scenario: A repo mori doesn't manage is refused
    When I run "mori tree create github.com/acme/gadget --agent claude --task fix-login"
    Then it fails with status NOT_FOUND and reason "REPO_NOT_MANAGED"

  Scenario: The same task twice is refused
    Given I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    When I run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    Then it fails with status ALREADY_EXISTS and reason "TREE_EXISTS"
    And no file or directory was created or modified

  Scenario: A workspace mori didn't make keeps its name
    Given someone added a workspace "claude-fix-login" to the clone outside mori
    When I run "mori tree create github.com/acme/widget --agent claude --task fix-login"
    Then it fails with status ALREADY_EXISTS and reason "WORKSPACE_EXISTS"

  Scenario: A task slug can't escape the trees directory
    When I run "mori tree create github.com/acme/widget --agent claude --task ../escape"
    Then it fails with status INVALID_ARGUMENT and reason "TREE_NAME_INVALID"

  Scenario: A revision that doesn't exist leaves nothing behind
    When I run "mori tree create github.com/acme/widget --agent claude --task fix-login --from nope"
    Then it fails with status INTERNAL and reason "JJ_FAILED"
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"
    And the clone has no workspace "claude-fix-login"

  Scenario: Dry run creates nothing
    When I run "mori tree create --dry-run github.com/acme/widget --agent claude --task fix-login"
    Then it succeeds
    And nothing exists at "<home>/mori/trees/widget/claude-fix-login"
    And the clone has no workspace "claude-fix-login"

  Scenario: JSON output for agents
    When I run "mori tree create --json github.com/acme/widget --agent claude --task fix-login"
    Then stdout is exactly one JSON object
    And it names the tree "claude-fix-login", its path and its lifetime "task-done"

  Scenario: A tree has a purpose, which tree set can change
    Given I have run "mori tree create widget --owner claude --task auth --purpose 'OAuth login'"
    And I have run "mori tree set widget claude-auth --purpose 'OAuth device flow' --lifetime pinned"
    When I run "mori ls --json"
    Then the forest shows "claude-auth" with purpose "OAuth device flow"
    And the forest shows "claude-auth" as claude's task "auth", lifetime "pinned"

  Scenario: tree set needs something to change
    Given I have run "mori tree create widget --owner claude --task auth"
    When I run "mori tree set widget claude-auth"
    Then it fails with status INVALID_ARGUMENT and reason "UPDATE_MASK_INVALID"

  Scenario: A purpose is one line
    When I run "mori tree create widget --owner claude --task auth --purpose ''"
    Then it fails with status INVALID_ARGUMENT and reason "PURPOSE_INVALID"

  Scenario: where inside a tree names its repo, record and context folder
    Given I have run "mori tree create widget --owner claude --task auth --purpose 'OAuth login'"
    When I run "mori where mori/trees/widget/claude-auth/src --json"
    Then it succeeds
    And where says tree "claude-auth" of "github.com/acme/widget", purpose "OAuth login"

  Scenario: where in the clone is its base tree
    When I run "mori where mori/repos/github.com/acme/widget --json"
    Then it succeeds
    And where says the base tree "default" of "github.com/acme/widget"

  Scenario: where in a tree directory mori didn't make reports it as foreign
    Given "<home>/mori/trees/widget/someone-elses/notes.txt" exists
    When I run "mori where mori/trees/widget/someone-elses --json"
    Then it succeeds
    And where says the tree "someone-elses" is foreign

  Scenario: where outside the root is nowhere
    When I run "mori where /"
    Then it fails with status NOT_FOUND and reason "NOT_IN_FOREST"

