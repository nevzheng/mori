Feature: Skills in the root
  mori puts its agent skills and an llms.txt index in the root, so any agent working there can
  learn to use mori. It only ever changes files it wrote and nobody edited since, and never
  touches anyone else's skills.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set

  Scenario: init installs mori's skills and the indexes
    When I run "mori init"
    Then it succeeds
    And "<home>/mori/skills/using-mori/SKILL.md" exists
    And "<home>/mori/skills/llms.txt" lists the skills "lead-tree" and "using-mori"
    And "<home>/mori/llms.txt" points to "skills/llms.txt"

  Scenario: sync updates a skill an older mori wrote and nobody edited
    Given I have run "mori init"
    And an older mori wrote "using-mori"
    When I run "mori skills sync"
    Then it succeeds
    And the output says "using-mori" was updated
    And "using-mori" is this mori's version

  Scenario: sync keeps a skill someone edited
    Given I have run "mori init"
    And someone edited the skill "using-mori"
    When I run "mori skills sync"
    Then it succeeds
    And the output says "using-mori" was kept
    And the skill "using-mori" still has the edit

  Scenario: a person's own skill is indexed and never touched
    Given I have run "mori init"
    And someone added their own skill "my-flow"
    When I run "mori skills sync"
    Then it succeeds
    And "<home>/mori/skills/llms.txt" lists the skills "my-flow" and "using-mori"
    And nothing under "<home>/mori/skills/my-flow" changed

  Scenario: a second sync changes nothing
    Given I have run "mori init"
    When I run "mori skills sync"
    Then it succeeds
    And no file or directory was created or modified

  Scenario: dry run writes nothing
    Given I have run "mori init"
    And an older mori wrote "using-mori"
    When I run "mori skills sync --dry-run"
    Then it succeeds
    And the output says "using-mori" would be updated
    And no file or directory was created or modified

  Scenario: init and ls say when the skills are from another mori
    Given I have run "mori init"
    And the skills were installed by mori "0.0.0-older"
    When I run "mori ls"
    Then it succeeds
    And it says the skills are from mori "0.0.0-older"

  Scenario: mori must be set up first
    When I run "mori skills sync"
    Then it fails with status FAILED_PRECONDITION and reason "NOT_INITIALIZED"
