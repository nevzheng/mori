Feature: CUJ 10 - see what the forest costs
  Agents can make as many trees as they like, so mori makes the cost visible. `mori ls --size`
  shows how much disk each tree uses, every `ls` says how much is free, and mori warns when free
  space drops below a floor (10% by default). Warnings never stop a command.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone github.com/acme/widget"
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-login"

  Scenario: Sizes show how much each tree uses
    Given "claude-fix-login" holds a 2 MB file
    When I run "mori ls --size --json"
    Then it succeeds
    And the forest shows "claude-fix-login" using at least 2 MB
    And the forest shows the disk's free and total space

  Scenario: Plain ls doesn't measure
    When I run "mori ls --json"
    Then it succeeds
    And the forest shows no sizes
    And the forest shows the disk's free and total space

  Scenario: A size measured moments ago is reused until asked to measure again
    Given I have run "mori ls --size"
    And "claude-fix-login" holds a 2 MB file
    When I run "mori ls --size --json"
    Then the forest shows "claude-fix-login" using less than 2 MB
    When I run "mori ls --size --fresh --json"
    Then the forest shows "claude-fix-login" using at least 2 MB

  Scenario: The text view adds a size column and a summary
    When I run "mori ls --size"
    Then it succeeds
    And the output has a SIZE column
    And the output says how much the trees use and how much is free

  Scenario: Low free space is called out
    Given config.toml sets the free-space floor to "100%"
    When I run "mori ls"
    Then it succeeds
    And the output warns that free space is below the "100%" floor

  Scenario: Low free space warns but still creates the tree
    Given config.toml sets the free-space floor to "100%"
    When I run "mori tree create github.com/acme/widget --agent claude --task fix-signup"
    Then it succeeds
    And "<home>/mori/trees/widget/claude-fix-signup" exists
    And the output warns that free space is below the "100%" floor

  Scenario: gc shows what each removable tree frees
    Given "claude-fix-login" landed a 2 MB file
    When I run "mori gc --offline --json"
    Then it succeeds
    And gc shows "claude-fix-login" freeing at least 2 MB

  Scenario: gc picks just enough safe trees to free a target, and removes only those
    Given "claude-fix-login" landed a 2 MB file
    And I have run "mori tree create github.com/acme/widget --agent claude --task fix-signup"
    And "claude-fix-signup" landed a 2 MB file
    When I run "mori gc --offline --free 1M --json"
    Then it succeeds
    And gc would remove 1 tree
    And "<home>/mori/trees/widget/claude-fix-login" exists
    When I run "mori gc --offline --free 1M --apply --yes"
    Then it succeeds
    And only one of "claude-fix-login" and "claude-fix-signup" is left
