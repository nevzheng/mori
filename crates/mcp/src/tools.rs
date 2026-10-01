//! The tools: their definitions, and each one's call into `mori-app`.

use mori_api::v1alpha1::ListTreesResponse;
use mori_app::{App, Backend, doctor, ls, place, tree, tree_set};
use mori_core::error::ErrorDetails;
use mori_core::tree::Lifetime;
use mori_core::vcs::Forge;
use serde_json::{Map, Value, json};

/// A JSON-RPC error: code and message.
type RpcError = (i64, String);

/// Every tool, as `tools/list` returns them.
pub fn definitions() -> Value {
    let read_only = json!({ "readOnlyHint": true, "openWorldHint": false });
    let writes = |idempotent: bool| {
        json!({
            "readOnlyHint": false,
            "destructiveHint": false,
            "idempotentHint": idempotent,
            "openWorldHint": false,
        })
    };
    json!([
        {
            "name": "mori_where",
            "title": "Where am I",
            "description": "Which repo and tree a path is in: the tree's owner, task, lifetime \
                and purpose, and the repo's context folder. Reads mori's records only.",
            "inputSchema": object(&[("path", string("An absolute path. Defaults to the \
                server's working directory."))], &[]),
            "annotations": read_only,
        },
        {
            "name": "mori_projects",
            "title": "Projects",
            "description": "The repos mori manages: path, VCS, context folder, and how many \
                trees each has and how many hold unsaved work.",
            "inputSchema": object(&[("query", string("Only repos whose name contains this, \
                ignoring case."))], &[]),
            "annotations": read_only,
        },
        {
            "name": "mori_trees",
            "title": "Trees",
            "description": "Trees with their owner, task, lifetime, purpose and work state \
                (edited, unpushed, landed). Filters combine.",
            "inputSchema": object(&[
                ("repo", string("Only this repo: full name or unique short name.")),
                ("query", string("Text in the repo, name, task or purpose, any case.")),
                ("owner", string("Only trees with this owner.")),
                ("status", json!({ "type": "string",
                    "enum": ["unsaved", "clean", "landed", "missing", "foreign"] })),
            ], &[]),
            "annotations": read_only,
        },
        {
            "name": "mori_tree_create",
            "title": "Create a tree",
            "description": "Gives one task its own tree, on top of trunk, recorded with its \
                owner, task, lifetime and purpose. Returns the path to work in.",
            "inputSchema": object(&[
                ("repo", string("The repo: full name or unique short name.")),
                ("task", string("A short slug: lowercase letters, digits and hyphens.")),
                ("owner", string("Who the tree is for, e.g. claude.")),
                ("purpose", string("What the tree is for, in one line.")),
                ("lifetime", string("pinned, task-done, lru, or ttl:<n>d.")),
                ("from", string("The revision to start from. Defaults to trunk.")),
            ], &["repo", "task", "owner", "purpose"]),
            "annotations": writes(false),
        },
        {
            "name": "mori_tree_set",
            "title": "Describe a tree",
            "description": "Changes a tree's purpose, lifetime or owner in mori's records. \
                Never touches the tree's files or the VCS.",
            "inputSchema": object(&[
                ("repo", string("The repo: full name or unique short name.")),
                ("name", string("The tree's name.")),
                ("purpose", string("What the tree is for, in one line; empty clears it.")),
                ("lifetime", string("pinned, task-done, lru, or ttl:<n>d.")),
                ("owner", string("Who the tree is for.")),
            ], &["repo", "name"]),
            "annotations": writes(true),
        },
        {
            "name": "mori_doctor",
            "title": "Check the forest",
            "description": "Drift between mori's records, the VCS and the disk, each finding \
                with the command that fixes it. Reads only; repairs are the person's call \
                (`mori doctor --fix --yes`).",
            "inputSchema": object(&[("repo", string("Only this repo."))], &[]),
            "annotations": read_only,
        },
    ])
}

fn string(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

fn object(properties: &[(&str, Value)], required: &[&str]) -> Value {
    let properties: Map<String, Value> = properties
        .iter()
        .map(|(name, schema)| ((*name).to_owned(), schema.clone()))
        .collect();
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

/// Runs `tools/call`: the named tool with its arguments.
pub fn call<V: Backend, F: Forge>(app: &App<V, F>, params: &Value) -> Result<Value, RpcError> {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = Args(params.get("arguments").cloned().unwrap_or(json!({})));
    let result = match name {
        "mori_where" => {
            let path = match args.string("path") {
                Some(path) => std::path::PathBuf::from(path),
                None => std::env::current_dir().map_err(|error| (-32603, error.to_string()))?,
            };
            if !path.is_absolute() {
                return Err((-32602, "path must be absolute".to_owned()));
            }
            to_value(place::run(app, &path))
        }
        "mori_projects" => {
            let listed = ls::run(app, &ls::LsArgs::default());
            match listed {
                Ok(listed) => Ok(projects(&listed, args.string("query").as_deref())),
                Err(error) => Err(status(error.as_ref())),
            }
        }
        "mori_trees" => {
            let list = ls::LsArgs {
                repo: args.string("repo"),
                query: args.string("query"),
                owner: args.string("owner"),
                status: args.status()?,
                ..ls::LsArgs::default()
            };
            to_value(ls::run(app, &list))
        }
        "mori_tree_create" => {
            let create = tree::CreateArgs {
                repo: args.required("repo")?,
                task: args.required("task")?,
                agent: Some(args.required("owner")?),
                lifetime: args.lifetime()?,
                from: args.string("from"),
                purpose: Some(args.required("purpose")?),
                dry_run: false,
            };
            to_value(tree::create(app, create))
        }
        "mori_tree_set" => {
            let set = tree_set::SetArgs {
                repo: args.required("repo")?,
                name: args.required("name")?,
                purpose: args.string("purpose"),
                lifetime: args.lifetime()?,
                owner: args.string("owner"),
            };
            to_value(tree_set::run(app, set))
        }
        "mori_doctor" => to_value(doctor::run(app, args.string("repo").as_deref(), None)),
        _ => return Err((-32602, format!("no tool {name:?}"))),
    };
    Ok(match result {
        Ok(value) => json!({
            "content": [{ "type": "text", "text": value.to_string() }],
            "structuredContent": value,
        }),
        Err(status) => json!({
            "content": [{ "type": "text", "text": status.to_string() }],
            "structuredContent": status,
            "isError": true,
        }),
    })
}

/// A response as JSON, or its error as a `google.rpc.Status`.
fn to_value<T: serde::Serialize>(result: Result<T, Box<dyn ErrorDetails>>) -> Result<Value, Value> {
    match result {
        Ok(response) => serde_json::to_value(response).map_err(
            |error| json!({ "code": 13, "message": format!("can't encode the response: {error}") }),
        ),
        Err(error) => Err(status(error.as_ref())),
    }
}

/// An error as `google.rpc.Status` JSON, the same shape the CLI prints with `--json`.
fn status(error: &dyn ErrorDetails) -> Value {
    let metadata: Map<String, Value> = error
        .metadata()
        .into_iter()
        .map(|(key, value)| (key.to_owned(), Value::String(value)))
        .collect();
    json!({
        "code": error.code().number(),
        "message": error.to_string(),
        "details": [{
            "@type": "type.googleapis.com/google.rpc.ErrorInfo",
            "reason": error.reason(),
            "domain": error.domain(),
            "metadata": metadata,
        }],
    })
}

/// The project map: each repo with counts, not every tree.
fn projects(listed: &ListTreesResponse, query: Option<&str>) -> Value {
    let query = query.map(str::to_lowercase);
    let repos: Vec<Value> = listed
        .repos
        .iter()
        .filter(|repo| {
            query
                .as_deref()
                .is_none_or(|query| repo.repo.to_lowercase().contains(query))
        })
        .map(|repo| {
            let unsaved = repo
                .trees
                .iter()
                .filter(|row| {
                    row.state
                        .as_ref()
                        .is_some_and(|state| state.changed || state.unpushed > 0)
                })
                .count();
            json!({
                "repo": repo.repo,
                "path": repo.path,
                "vcs": repo.vcs().as_str_name(),
                "contextDir": repo.context_dir,
                "trees": repo.trees.len(),
                "unsaved": unsaved,
            })
        })
        .collect();
    json!({ "repos": repos })
}

/// A tool call's arguments.
struct Args(Value);

impl Args {
    fn string(&self, name: &str) -> Option<String> {
        self.0.get(name).and_then(Value::as_str).map(str::to_owned)
    }

    fn required(&self, name: &str) -> Result<String, RpcError> {
        self.string(name)
            .ok_or_else(|| (-32602, format!("{name} is required")))
    }

    fn lifetime(&self) -> Result<Option<Lifetime>, RpcError> {
        self.string("lifetime")
            .map(|lifetime| lifetime.parse().map_err(|why| (-32602, why)))
            .transpose()
    }

    fn status(&self) -> Result<Option<ls::StatusFilter>, RpcError> {
        Ok(match self.string("status").as_deref() {
            None => None,
            Some("unsaved") => Some(ls::StatusFilter::Unsaved),
            Some("clean") => Some(ls::StatusFilter::Clean),
            Some("landed") => Some(ls::StatusFilter::Landed),
            Some("missing") => Some(ls::StatusFilter::Missing),
            Some("foreign") => Some(ls::StatusFilter::Foreign),
            Some(other) => return Err((-32602, format!("no status {other:?}"))),
        })
    }
}
