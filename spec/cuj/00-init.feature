@wip
Feature: CUJ 0 - set up mori
  Once per machine, `mori init` creates the root layout, the config and the state store.
  It only ever adds things, and running it twice is safe.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set

  Scenario: First run creates everything under the default root
    When I run "mori init"
    Then it succeeds
    And "<home>/mori/repos/" and "<home>/mori/trees/" exist
    And "$XDG_CONFIG_HOME/mori/config.toml" records the root "<home>/mori"
    And "$XDG_STATE_HOME/mori/" exists with mode 0700
    And the database exists with mode 0600 and records the root "<home>/mori"
    And the output lists every path it created

  Scenario: MORI_ROOT chooses the root
    Given MORI_ROOT is "<home>/elsewhere/mori"
    When I run "mori init"
    Then it succeeds
    And "<home>/elsewhere/mori/repos/" exists
    And nothing exists at "<home>/mori"

  Scenario: Second run changes nothing
    Given I have run "mori init"
    When I run "mori init"
    Then it succeeds
    And no file or directory was created or modified
    And the output says mori is already set up

  Scenario: Dry run touches nothing
    When I run "mori init --dry-run"
    Then it succeeds
    And the output lists the paths it would create
    And nothing new exists under <home>

  Scenario: A changed root is a misconfiguration
    Given I have run "mori init"
    And MORI_ROOT is "<home>/other"
    When I run "mori init"
    Then it fails with status FAILED_PRECONDITION and reason "ROOT_MISMATCH"
    And the message says moving a root needs a migration that isn't defined yet
    And nothing exists at "<home>/other"

  Scenario: Existing clones are left alone and reported as unmanaged
    Given "<home>/mori/repos/github.com/acme/widget/.git" exists
    When I run "mori init"
    Then it succeeds
    And nothing under "<home>/mori/repos/github.com/acme/widget" changed
    And the output lists "github.com/acme/widget" as unmanaged

  Scenario: JSON output for agents
    When I run "mori init --json"
    Then stdout is exactly one JSON object
    And it lists the root, the created paths and the unmanaged repos
