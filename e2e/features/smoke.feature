Feature: Harness smoke test
  Proves the end-to-end harness works: Bazel builds mori, and pytest runs it in a clean home.
  Not a CUJ; it only covers what the binary can already do.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it

  Scenario: The binary reports its version
    When I run "mori --version"
    Then it succeeds
    And stdout matches "mori \d+\.\d+\.\d+"

  Scenario: Unknown arguments are a usage error
    When I run "mori plant a-tree"
    Then it fails with exit code 3
