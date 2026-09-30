Feature: CUJ 1 - add a repo
  `mori clone` puts one clone of a repo at repos/<host>/<owner>/<repo> and records it. It is the
  VCS clone plus a record of it; mori only ever manages clones it made.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"

  Scenario: Cloning puts the repo in the layout and records it
    When I run "mori clone github.com/acme/widget"
    Then it succeeds
    And "<home>/mori/repos/github.com/acme/widget/README.md" exists
    And the clone is a jj repo colocated with git
    And mori records "github.com/acme/widget" with a pinned base tree
    And the output says it cloned "github.com/acme/widget"

  Scenario: A full URL names the same repo as the short form
    When I run "mori clone https://github.com/acme/widget.git"
    Then it succeeds
    And mori records "github.com/acme/widget" with a pinned base tree

  Scenario: mori init doesn't call mori's own clones unmanaged
    Given I have run "mori clone github.com/acme/widget"
    When I run "mori init"
    Then it succeeds
    And the output says mori is already set up
    And the output lists no unmanaged repos

  Scenario: A jj-only clone
    When I run "mori clone --no-colocate github.com/acme/widget"
    Then it succeeds
    And the clone is a jj repo without git

  Scenario: A repo mori already has is refused
    Given I have run "mori clone github.com/acme/widget"
    When I run "mori clone git@github.com:acme/widget.git"
    Then it fails with status ALREADY_EXISTS and reason "REPO_EXISTS"
    And no file or directory was created or modified

  Scenario: A directory mori didn't make is left alone
    Given "<home>/mori/repos/github.com/acme/widget/notes" exists
    When I run "mori clone github.com/acme/widget"
    Then it fails with status ALREADY_EXISTS and reason "PATH_EXISTS"
    And nothing under "<home>/mori/repos/github.com/acme/widget" changed
    And mori records no repos

  Scenario: A failed clone leaves nothing behind
    When I run "mori clone github.com/acme/missing"
    Then it fails with status INTERNAL and reason "JJ_FAILED"
    And nothing exists at "<home>/mori/repos/github.com/acme/missing"
    And mori records no repos

  Scenario: If recording fails, the clone is kept
    Given mori's database is read-only
    When I run "mori clone github.com/acme/widget"
    Then it fails with status INTERNAL and reason "CLONE_NOT_RECORDED"
    And "<home>/mori/repos/github.com/acme/widget/README.md" exists
    And the message says the clone is kept

  Scenario: Dry run clones nothing
    When I run "mori clone --dry-run github.com/acme/widget"
    Then it succeeds
    And nothing exists at "<home>/mori/repos/github.com/acme/widget"
    And mori records no repos

  Scenario: A URL mori can't use is refused
    When I run "mori clone http://github.com/acme/widget"
    Then it fails with status INVALID_ARGUMENT and reason "CLONE_URL_INVALID"

  Scenario: JSON output for agents
    When I run "mori clone --json github.com/acme/widget"
    Then stdout is exactly one JSON object
    And it names the repo "github.com/acme/widget", its path and that it is colocated

  Scenario: mori must be set up first
    Given mori's database is gone
    When I run "mori clone github.com/acme/widget"
    Then it fails with status FAILED_PRECONDITION and reason "NOT_INITIALIZED"
