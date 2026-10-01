//! Hints: what mori noticed that the reader may want to act on, and the command that does it.
//!
//! Each hint fires only for whoever it is about: the agent-directed ones need `MORI_AGENT`, so a
//! person never sees them. Messages are fixed templates that interpolate only names mori validated
//! (repos, trees) and counts, never a purpose or other free text, so a hint can't carry
//! instructions someone planted.

use mori_api::v1alpha1::{
    Hint, ListTreesResponse, ResolveResponse, Tree, resolve_response::Kind, tree_row::Status,
};
use mori_core::clone::BASE_TREE_NAME;

use crate::Host;

/// A command gives at most this many hints, so they stay worth reading.
const MAX: usize = 2;

/// `hints` as the host allows them: none with `MORI_HINTS=0`, else the first [`MAX`].
pub(crate) fn allowed(host: &Host, mut hints: Vec<Hint>) -> Vec<Hint> {
    if host.hints_off {
        return Vec::new();
    }
    hints.truncate(MAX);
    hints
}

/// For `mori where`: an agent in the person's clone, in someone else's tree, or in its own tree
/// with no purpose.
pub(crate) fn for_where(agent: Option<&str>, response: &ResolveResponse) -> Vec<Hint> {
    let (Some(agent), Some(tree)) = (agent, &response.tree) else {
        return Vec::new();
    };
    let repo = &response.repo;
    let create = format!("mori tree create {repo} --task <task> --purpose \"<what for>\"");
    match response.kind() {
        Kind::Clone => vec![hint(
            "IN_PERSONS_ROOT",
            "this is the clone itself, a person's checkout; an agent works in a tree of its own"
                .to_owned(),
            create,
            tree,
        )],
        Kind::Tree if response.status() == Status::Tree && tree.owner != agent => vec![hint(
            "NOT_YOUR_TREE",
            format!(
                "tree {} belongs to another owner; work in a tree of your own",
                tree.name
            ),
            create,
            tree,
        )],
        Kind::Tree if response.status() == Status::Tree => no_purpose(tree).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// For `mori tree create`: an agent's tree made without a purpose. A person's own tree needs none.
pub(crate) fn for_create(user: Option<&str>, tree: &Tree) -> Vec<Hint> {
    if user == Some(tree.owner.as_str()) {
        return Vec::new();
    }
    no_purpose(tree).into_iter().collect()
}

/// For `mori ls` and the dashboard: trees whose work landed, which `mori gc` may free. Pinned
/// trees don't count: gc leaves them.
pub(crate) fn for_ls(response: &ListTreesResponse) -> Vec<Hint> {
    let landed = response
        .repos
        .iter()
        .flat_map(|repo| &repo.trees)
        .filter(|row| {
            row.status() == Status::Tree
                && row
                    .tree
                    .as_ref()
                    .is_some_and(|tree| tree.lifetime != "pinned")
                && row.bookmarks.iter().any(|bookmark| bookmark.landed)
        })
        .count();
    if landed == 0 {
        return Vec::new();
    }
    let trees = if landed == 1 { "tree" } else { "trees" };
    vec![Hint {
        code: "LANDED_TREES".to_owned(),
        message: format!("{landed} {trees} hold work that has landed; gc shows which can go"),
        command: "mori gc".to_owned(),
        ..Hint::default()
    }]
}

fn no_purpose(tree: &Tree) -> Option<Hint> {
    (tree.purpose.is_empty() && tree.name != BASE_TREE_NAME).then(|| {
        hint(
            "NO_PURPOSE",
            format!(
                "tree {} has no purpose; one line on what it is for shows in ls and where",
                tree.name
            ),
            format!(
                "mori tree set {} {} --purpose \"<what for>\"",
                tree.repo, tree.name
            ),
            tree,
        )
    })
}

fn hint(code: &str, message: String, command: String, tree: &Tree) -> Hint {
    Hint {
        code: code.to_owned(),
        message,
        command,
        repo: tree.repo.clone(),
        tree: tree.name.clone(),
    }
}
