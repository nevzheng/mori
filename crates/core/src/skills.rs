//! Skills in the root: what mori writes into `skills/` and the two `llms.txt` indexes, and what it
//! leaves alone.
//!
//! mori only changes what it wrote. The manifest records the hash of every file mori wrote; a
//! file that still matches is mori's to update, one that doesn't was edited and is kept, and one
//! mori never wrote is someone else's. The adapter reads the disk and the manifest, [`plan`]
//! decides, and the adapter writes.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

/// A file mori wants in the root, as a path relative to the root and its contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wanted {
    /// Relative to the root, e.g. `skills/using-mori/SKILL.md`.
    pub path: String,
    /// What mori would write.
    pub contents: String,
}

/// What mori wrote before: relative path to the SHA-256 of the contents it wrote.
pub type Manifest = BTreeMap<String, String>;

/// What `sync` does with one file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// It wasn't there: write it.
    Install,
    /// It is already exactly what mori would write.
    Unchanged,
    /// mori wrote it and nobody changed it since: rewrite it.
    Update,
    /// mori wrote it, but someone changed it since: keep their version.
    KeptEdited,
    /// It is there, but mori never wrote it: someone else's.
    SkippedNotOurs,
}

impl Action {
    /// True if the file gets written.
    #[must_use]
    pub fn writes(self) -> bool {
        matches!(self, Self::Install | Self::Update)
    }
}

/// One file's fate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The file.
    pub wanted: Wanted,
    /// What happens to it.
    pub action: Action,
}

/// Whether this run may change existing files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// `mori init`: only adds files that are missing.
    AddOnly,
    /// `mori skills sync`: also updates files mori wrote that nobody changed.
    Sync,
}

/// The SHA-256 of `contents`, in lowercase hex.
#[must_use]
pub fn hash(contents: &str) -> String {
    Sha256::digest(contents.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// Decides each wanted file's fate from what is on disk (`on_disk`: relative path to the hash of
/// its contents, for the wanted files that exist) and what mori wrote before.
#[must_use]
pub fn plan(
    wanted: Vec<Wanted>,
    on_disk: &BTreeMap<String, String>,
    manifest: &Manifest,
    mode: Mode,
) -> Vec<Step> {
    wanted
        .into_iter()
        .map(|wanted| {
            let action = match (on_disk.get(&wanted.path), manifest.get(&wanted.path)) {
                (None, _) => Action::Install,
                (Some(disk), _) if *disk == hash(&wanted.contents) => Action::Unchanged,
                (Some(_), None) => Action::SkippedNotOurs,
                (Some(disk), Some(written)) if disk == written => match mode {
                    Mode::Sync => Action::Update,
                    Mode::AddOnly => Action::Unchanged,
                },
                (Some(_), Some(_)) => Action::KeptEdited,
            };
            Step { wanted, action }
        })
        .collect()
}

/// A skill's name and description, from the frontmatter of its `SKILL.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillInfo {
    /// Its directory under `skills/`.
    pub dir: String,
    /// The frontmatter `name`, or the directory name.
    pub name: String,
    /// The frontmatter `description`, on one line.
    pub description: String,
}

/// Reads `name` and `description` from `SKILL.md` frontmatter: plain values, and folded (`>`,
/// `>-`) or literal (`|`) blocks, which are joined onto one line. Anything else is ignored.
#[must_use]
pub fn skill_info(dir: &str, skill_md: &str) -> SkillInfo {
    let mut info = SkillInfo {
        dir: dir.to_owned(),
        name: dir.to_owned(),
        description: String::new(),
    };
    let mut lines = skill_md.lines();
    if lines.next().map(str::trim) != Some("---") {
        return info;
    }
    let body: Vec<&str> = lines.take_while(|line| line.trim() != "---").collect();
    let mut index = 0;
    while let Some(line) = body.get(index) {
        index += 1;
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let value = value.trim();
        let value = if matches!(value, ">" | ">-" | "|" | "|-") {
            let mut block = Vec::new();
            while let Some(next) = body.get(index) {
                if !next.starts_with(char::is_whitespace) && !next.trim().is_empty() {
                    break;
                }
                block.push(next.trim());
                index += 1;
            }
            block.join(" ").trim().to_owned()
        } else {
            value.trim_matches(|c| c == '"' || c == '\'').to_owned()
        };
        match key.trim() {
            "name" if !value.is_empty() => info.name = value,
            "description" => info.description = value,
            _ => {}
        }
    }
    info
}

/// The contents of `skills/llms.txt`: every skill, sorted by name.
#[must_use]
pub fn skills_index(skills: &[SkillInfo]) -> String {
    let mut skills = skills.to_vec();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    let mut text = String::from(
        "# Skills\n\n\
         > Skills for agents working under this root. Each is a directory, `<name>/SKILL.md`, with\n\
         > a `name` and a `description` in its frontmatter. mori's own skills and anyone else's sit\n\
         > side by side; mori generates this index and updates only the skills it wrote.\n\n\
         ## Skills\n\n",
    );
    for skill in &skills {
        let _ = writeln!(
            text,
            "- [{}]({}/SKILL.md): {}",
            skill.name, skill.dir, skill.description
        );
    }
    text
}

/// The contents of the root `llms.txt`.
#[must_use]
pub fn root_index() -> String {
    "# mori\n\n\
     > This directory is a mori root: one clone per repo, and one tree per piece of work, for\n\
     > people and their coding agents. mori records only what it created. Pre-alpha: anything may\n\
     > change in any release.\n\n\
     Layout:\n\n\
     - `repos/<host>/<owner>/<repo>/`: one clone per repo, managed by `mori clone`\n\
     - `trees/<repo>/<name>/`: task trees, managed by `mori tree create` and `mori tree remove`\n\
     - `skills/`: skills for agents, indexed in `skills/llms.txt`\n\n\
     ## Start here\n\n\
     - [Skills index](skills/llms.txt): how to use mori, and every other skill in this root\n\
     - [mori](https://nevzheng.github.io/mori/): the project's site and design docs\n"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wanted(contents: &str) -> Wanted {
        Wanted {
            path: "skills/using-mori/SKILL.md".to_owned(),
            contents: contents.to_owned(),
        }
    }

    fn action(on_disk: Option<&str>, written: Option<&str>, mode: Mode) -> Action {
        let path = "skills/using-mori/SKILL.md".to_owned();
        let on_disk: BTreeMap<_, _> = on_disk
            .map(|text| (path.clone(), hash(text)))
            .into_iter()
            .collect();
        let manifest: Manifest = written
            .map(|text| (path.clone(), hash(text)))
            .into_iter()
            .collect();
        plan(vec![wanted("new")], &on_disk, &manifest, mode)[0].action
    }

    #[test]
    fn every_row_of_the_rule() {
        assert_eq!(action(None, None, Mode::Sync), Action::Install);
        assert_eq!(action(None, Some("old"), Mode::Sync), Action::Install);
        assert_eq!(action(Some("new"), None, Mode::Sync), Action::Unchanged);
        assert_eq!(action(Some("old"), Some("old"), Mode::Sync), Action::Update);
        assert_eq!(
            action(Some("edited"), Some("old"), Mode::Sync),
            Action::KeptEdited
        );
        assert_eq!(
            action(Some("theirs"), None, Mode::Sync),
            Action::SkippedNotOurs
        );
    }

    #[test]
    fn init_only_adds() {
        assert_eq!(action(None, None, Mode::AddOnly), Action::Install);
        assert_eq!(
            action(Some("old"), Some("old"), Mode::AddOnly),
            Action::Unchanged
        );
        assert!(!action(Some("old"), Some("old"), Mode::AddOnly).writes());
    }

    #[test]
    fn hashes_are_sha256_hex() {
        assert_eq!(
            hash(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn frontmatter_gives_name_and_description() {
        let folded =
            "---\nname: lead-tree\ndescription: >-\n  How to run\n  a lead tree.\n---\n\n# Body\n";
        let plain = "---\nname: \"my-flow\"\ndescription: My review flow.\nother: x\n---\n";
        let none = "# Just a heading\n";

        assert_eq!(
            skill_info("lead-tree", folded),
            SkillInfo {
                dir: "lead-tree".to_owned(),
                name: "lead-tree".to_owned(),
                description: "How to run a lead tree.".to_owned(),
            }
        );
        assert_eq!(skill_info("x", plain).name, "my-flow");
        assert_eq!(skill_info("x", plain).description, "My review flow.");
        assert_eq!(skill_info("bare", none).name, "bare");
        assert_eq!(skill_info("bare", none).description, "");
    }

    #[test]
    fn the_skills_index_lists_every_skill_by_name() {
        let index = skills_index(&[
            skill_info(
                "using-mori",
                "---\nname: using-mori\ndescription: Use mori.\n---\n",
            ),
            skill_info("a-flow", "---\nname: a-flow\ndescription: Mine.\n---\n"),
        ]);

        let lines: Vec<&str> = index
            .lines()
            .filter(|line| line.starts_with("- ["))
            .collect();
        assert_eq!(
            lines,
            [
                "- [a-flow](a-flow/SKILL.md): Mine.",
                "- [using-mori](using-mori/SKILL.md): Use mori.",
            ]
        );
    }

    #[test]
    fn the_root_index_points_to_the_skills_index() {
        assert!(root_index().contains("(skills/llms.txt)"));
    }
}
