Feature: Harness smoke test
  Proves the end-to-end harness works: Bazel builds mori, and pytest runs it in a clean home.
  Not a CUJ; it only covers what the binary can already do.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it

  Scenario: The binary reports its version
    When I run "mori --version"
    Then it succeeds
    And stdout matches "mori \d+\.\d+\.\d+(-[0-9A-Za-z.]+)?"

  Scenario: Unknown arguments are a usage error
    When I run "mori plant a-tree"
    Then it fails with exit code 3

  Scenario: mori alone in a pipe prints the help
    When I run "mori"
    Then it fails with exit code 3
    And stdout matches "(?s)mori looks after a forest.*Usage: mori.*"

  Scenario: Colour never reaches a pipe or JSON
    When I run "mori --color auto --help"
    Then it succeeds
    And stdout has no colour codes

