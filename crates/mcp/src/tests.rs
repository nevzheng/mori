use std::path::Path;

use mori_app::{App, Backend, Host, init};
use mori_core::clone::RepoId;
use mori_core::error::{Code, ErrorDetails};
use mori_core::forest::{TreeState, Workspace, Workspaces};
use mori_core::paths::Env;
use mori_core::vcs::{Forge, Merged, RemoteBookmark, Vcs, VcsKind};
use serde_json::{Value, json};
use tempfile::TempDir;

use super::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// A VCS with no workspaces that refuses every change: enough for a root with no clones.
struct NoVcs;

#[derive(Debug)]
struct Refused;

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("refused")
    }
}

impl std::error::Error for Refused {}

impl ErrorDetails for Refused {
    fn code(&self) -> Code {
        Code::Internal
    }
    fn reason(&self) -> &'static str {
        "REFUSED"
    }
    fn domain(&self) -> &'static str {
        "test.mori"
    }
    fn metadata(&self) -> Vec<(&'static str, String)> {
        Vec::new()
    }
}

impl Workspaces for NoVcs {
    type Error = Refused;
    fn list(&self, _: &Path) -> std::result::Result<Vec<Workspace>, Refused> {
        Ok(Vec::new())
    }
    fn state(&self, _: &Path, _: &str) -> std::result::Result<TreeState, Refused> {
        Err(Refused)
    }
}

impl Vcs for NoVcs {
    fn clone_repo(&self, _: &str, _: &Path, _: bool) -> std::result::Result<(), Refused> {
        Err(Refused)
    }
    fn add_tree(&self, _: &Path, _: &str, _: &Path, _: &str) -> std::result::Result<(), Refused> {
        Err(Refused)
    }
    fn add_tree_at(
        &self,
        _: &Path,
        _: &str,
        _: &Path,
        _: &str,
    ) -> std::result::Result<(), Refused> {
        Err(Refused)
    }
    fn snapshot(&self, _: &Path) -> std::result::Result<(), Refused> {
        Ok(())
    }
    fn forget_tree(&self, _: &Path, _: &str) -> std::result::Result<(), Refused> {
        Err(Refused)
    }
    fn state_covering(
        &self,
        _: &Path,
        _: &str,
        _: &[String],
    ) -> std::result::Result<TreeState, Refused> {
        Err(Refused)
    }
    fn working_copy_commit(&self, _: &Path, _: &str) -> std::result::Result<String, Refused> {
        Err(Refused)
    }
    fn pushed_bookmarks(
        &self,
        _: &Path,
        _: &str,
    ) -> std::result::Result<Vec<RemoteBookmark>, Refused> {
        Ok(Vec::new())
    }
    fn last_change(&self, _: &Path, _: &str) -> std::result::Result<u64, Refused> {
        Ok(0)
    }
    fn pin(&self, _: &Path, _: &str, _: &str) -> std::result::Result<(), Refused> {
        Err(Refused)
    }
    fn commit_exists(&self, _: &Path, _: &str) -> std::result::Result<bool, Refused> {
        Ok(false)
    }
    fn fetch(&self, _: &Path) -> std::result::Result<(), Refused> {
        Ok(())
    }

    fn remote_bookmarks(&self, _: &Path) -> std::result::Result<Vec<RemoteBookmark>, Refused> {
        Ok(Vec::new())
    }
}

impl Backend for NoVcs {
    fn clone_as(&self, _: VcsKind, _: &str, _: &Path, _: bool) -> std::result::Result<(), Refused> {
        Err(Refused)
    }
}

struct NoForge;

impl Forge for NoForge {
    fn pr_merged(&self, _: &RepoId, _: &str) -> Merged {
        Merged::Unknown
    }
}

/// A set-up root with no clones, under a temporary HOME.
fn app() -> Result<(TempDir, App<NoVcs, NoForge>)> {
    let home = TempDir::new()?;
    let host = Host {
        env: Env {
            home: Some(home.path().to_path_buf()),
            xdg_config_home: Some(home.path().join(".config")),
            xdg_state_home: Some(home.path().join(".state")),
            xdg_cache_home: Some(home.path().join(".cache")),
            ..Env::default()
        },
        user: Some("tester".to_owned()),
        ..Host::default()
    };
    init::run(&host, false).map_err(|error| error.to_string())?;
    Ok((home, App::new(host, NoVcs, NoForge)))
}

/// Sends `requests`, one per line, and returns the responses.
fn exchange(app: &App<NoVcs, NoForge>, requests: &[Value]) -> Result<Vec<Value>> {
    let input = requests
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    let mut output = Vec::new();
    serve(app, input.as_bytes(), &mut output)?;
    Ok(String::from_utf8(output)?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?)
}

fn call(id: u32, name: &str, arguments: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": { "name": name, "arguments": arguments } })
}

#[test]
fn initialize_agrees_a_version_and_notifications_get_no_reply() -> Result<()> {
    let (_home, app) = app()?;

    let replies = exchange(
        &app,
        &[
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize",
                    "params": { "protocolVersion": "2025-03-26" } }),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "initialize",
                    "params": { "protocolVersion": "1999-01-01" } }),
        ],
    )?;

    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["result"]["protocolVersion"], "2025-03-26");
    assert_eq!(replies[0]["result"]["serverInfo"]["name"], "mori");
    assert_eq!(
        replies[1]["result"]["protocolVersion"],
        PROTOCOL_VERSIONS[0]
    );
    Ok(())
}

#[test]
fn the_tools_are_six_and_only_two_write() -> Result<()> {
    let (_home, app) = app()?;

    let replies = exchange(
        &app,
        &[json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" })],
    )?;

    let tools = replies[0]["result"]["tools"].as_array().ok_or("no tools")?;
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert_eq!(
        names,
        [
            "mori_where",
            "mori_projects",
            "mori_trees",
            "mori_tree_create",
            "mori_tree_set",
            "mori_doctor"
        ]
    );
    let writing: Vec<&str> = tools
        .iter()
        .filter(|tool| tool["annotations"]["readOnlyHint"] == false)
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert_eq!(writing, ["mori_tree_create", "mori_tree_set"]);
    assert!(
        tools
            .iter()
            .all(|tool| tool["annotations"]["destructiveHint"] != true)
    );
    Ok(())
}

#[test]
fn tools_answer_with_structured_content_or_a_status() -> Result<()> {
    let (home, app) = app()?;

    let replies = exchange(
        &app,
        &[
            call(1, "mori_projects", &json!({})),
            call(2, "mori_where", &json!({ "path": "/" })),
            call(
                3,
                "mori_where",
                &json!({ "path": home.path().join("mori").display().to_string() }),
            ),
            call(4, "mori_tree_set", &json!({ "repo": "widget" })),
            json!({ "jsonrpc": "2.0", "id": 5, "method": "nope" }),
        ],
    )?;

    assert_eq!(
        replies[0]["result"]["structuredContent"],
        json!({ "repos": [] })
    );
    assert_eq!(replies[1]["result"]["isError"], true);
    assert_eq!(
        replies[1]["result"]["structuredContent"]["details"][0]["reason"],
        "NOT_IN_FOREST"
    );
    assert_eq!(
        replies[2]["result"]["structuredContent"]["kind"],
        "KIND_ROOT"
    );
    assert_eq!(replies[3]["error"]["code"], -32602);
    assert_eq!(replies[4]["error"]["code"], -32601);
    Ok(())
}
