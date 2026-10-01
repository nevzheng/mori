Feature: CUJ 11 - agents ask mori over MCP
  `mori mcp` serves mori's tools to agents over stdio: where they are, the repos and trees with
  their purposes, a health check, and creating or describing a tree. Removing, collecting and
  repairing stay in the CLI, for the person.

  Background:
    Given a temporary HOME with XDG_CONFIG_HOME, XDG_STATE_HOME and XDG_CACHE_HOME inside it
    And MORI_ROOT is not set
    And "github.com/acme/widget" is a repo on the remote
    And I have run "mori init"
    And I have run "mori clone github.com/acme/widget"

  Scenario: An agent creates a tree with a purpose and finds itself in it
    When an agent calls over MCP:
      | tool             | arguments                                                                        |
      | mori_tree_create | {"repo": "widget", "task": "auth", "owner": "claude", "purpose": "OAuth login"}  |
      | mori_where       | {"path": "<home>/mori/trees/widget/claude-auth"}                                 |
      | mori_trees       | {"owner": "claude"}                                                              |
    Then every call succeeds
    And the "mori_where" answer names the tree "claude-auth" with purpose "OAuth login"
    And the "mori_trees" answer lists only "claude-auth"

  Scenario: The tools that remove things aren't served
    When an agent lists the MCP tools
    Then they are "mori_where, mori_projects, mori_trees, mori_tree_create, mori_tree_set, mori_doctor"
