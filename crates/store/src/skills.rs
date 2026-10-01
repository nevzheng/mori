//! mori's agent skills in the root: embedded in the binary, written into `context/skills/` with the two
//! `llms.txt` indexes, and tracked in a manifest so mori only ever changes what it wrote.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use mori_core::paths::Paths;
use mori_core::skills::{
    Manifest, Mode, RepoContext, SkillInfo, Step, Wanted, context_index, hash, plan, root_index,
    skill_info,
};
use serde::{Deserialize, Serialize};

use crate::StoreError;

/// mori's skills, by directory name: the repo's `skills/<name>/SKILL.md`, fixed at build time.
/// `skills/llms.txt` in the repo lists them; a test keeps the two in step.
pub const EMBEDDED: [(&str, &str); 4] = [
    (
        "agent-workflows",
        include_str!("../../../skills/agent-workflows/SKILL.md"),
    ),
    (
        "lead-tree",
        include_str!("../../../skills/lead-tree/SKILL.md"),
    ),
    (
        "using-mori",
        include_str!("../../../skills/using-mori/SKILL.md"),
    ),
    (
        "vcs-in-mori",
        include_str!("../../../skills/vcs-in-mori/SKILL.md"),
    ),
];

/// Where the two generated indexes go, relative to the root.
pub const CONTEXT_INDEX: &str = "context/llms.txt";
/// The root index, relative to the root.
pub const ROOT_INDEX: &str = "llms.txt";

/// Mode for the files mori writes into the root: readable by everyone, like any docs.
const FILE_MODE: u32 = 0o644;

/// The manifest's name in the state directory.
const MANIFEST: &str = "skills.json";

/// What the manifest file holds.
#[derive(Debug, Default, Deserialize, Serialize)]
struct ManifestFile {
    /// The mori version that last wrote skills.
    version: String,
    /// Relative path to the SHA-256 of what mori wrote there.
    files: Manifest,
}

/// What exists before a skills sync, as read from disk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observed {
    /// For each file mori might write that exists: relative path to the hash of its contents. A
    /// symlink gets a marker that matches no hash, so it is never written through.
    pub on_disk: BTreeMap<String, String>,
    /// What mori wrote before.
    pub manifest: Manifest,
    /// Every skill directory under `context/skills/` (not symlinks) that has a `SKILL.md`.
    pub skills: Vec<SkillInfo>,
}

/// The mori version that last wrote skills into the root, if any did and the manifest is
/// readable. Best effort: it only feeds a hint.
#[must_use]
pub fn installed_version(paths: &Paths) -> Option<String> {
    let text = std::fs::read_to_string(manifest_path(paths)).ok()?;
    serde_json::from_str::<ManifestFile>(&text)
        .ok()
        .map(|file| file.version)
}

/// The version of mori this binary is.
pub const THIS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the manifest lives: in mori's state directory, not in the root.
#[must_use]
pub fn manifest_path(paths: &Paths) -> PathBuf {
    paths.state_dir.join(MANIFEST)
}

fn skill_path(dir: &str) -> String {
    format!("context/skills/{dir}/SKILL.md")
}

/// Reads the disk and the manifest. Reads only.
///
/// # Errors
///
/// I/O errors other than "not found", and a manifest that isn't valid JSON.
pub fn observe(paths: &Paths) -> Result<Observed, StoreError> {
    let candidates = EMBEDDED
        .iter()
        .map(|(dir, _)| skill_path(dir))
        .chain([CONTEXT_INDEX.to_owned(), ROOT_INDEX.to_owned()]);
    let mut on_disk = BTreeMap::new();
    for relative in candidates {
        if let Some(hash) = file_hash(&paths.root.join(&relative))? {
            on_disk.insert(relative, hash);
        }
    }
    let manifest_path = paths.state_dir.join(MANIFEST);
    let manifest = match std::fs::read_to_string(&manifest_path) {
        Ok(text) => {
            serde_json::from_str::<ManifestFile>(&text)
                .map_err(|error| StoreError::Io {
                    path: manifest_path.clone(),
                    source: std::io::Error::new(ErrorKind::InvalidData, error),
                })?
                .files
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Manifest::new(),
        Err(source) => return Err(io(&manifest_path, source)),
    };
    Ok(Observed {
        on_disk,
        manifest,
        skills: skills_on_disk(&paths.root.join("context/skills"))?,
    })
}

/// The hash of the file at `path`; a marker for a symlink; none if nothing is there.
fn file_hash(path: &Path) -> Result<Option<String>, StoreError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(Some("symlink".to_owned())),
        Ok(_) => {
            let bytes = std::fs::read(path).map_err(|source| io(path, source))?;
            Ok(Some(hash(&String::from_utf8_lossy(&bytes))))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io(path, source)),
    }
}

fn skills_on_disk(skills: &Path) -> Result<Vec<SkillInfo>, StoreError> {
    let entries = match std::fs::read_dir(skills) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(io(skills, source)),
    };
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| io(skills, source))?;
        let file_type = entry
            .file_type()
            .map_err(|source| io(&entry.path(), source))?;
        let skill_md = entry.path().join("SKILL.md");
        if !file_type.is_dir() || !skill_md.is_file() {
            continue;
        }
        let dir = entry.file_name().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&skill_md).map_err(|source| io(&skill_md, source))?;
        found.push(skill_info(&dir, &text));
    }
    Ok(found)
}

/// Plans a sync: mori's skill files first, then the indexes, which list every skill as it will
/// be once the skill files are written.
#[must_use]
pub fn sync_plan(observed: &Observed, mode: Mode, repos: &[RepoContext]) -> Vec<Step> {
    let wanted = EMBEDDED
        .iter()
        .map(|(dir, contents)| Wanted {
            path: skill_path(dir),
            contents: (*contents).to_owned(),
        })
        .collect();
    let mut steps = plan(wanted, &observed.on_disk, &observed.manifest, mode);

    let written: BTreeSet<&str> = EMBEDDED
        .iter()
        .zip(&steps)
        .filter(|(_, step)| step.action.writes())
        .map(|((dir, _), _)| *dir)
        .collect();
    let mut skills: Vec<SkillInfo> = observed
        .skills
        .iter()
        .filter(|skill| !written.contains(skill.dir.as_str()))
        .cloned()
        .collect();
    for (dir, contents) in EMBEDDED {
        if written.contains(dir) {
            skills.push(skill_info(dir, contents));
        }
    }
    steps.extend(plan(
        indexes(&skills, repos),
        &observed.on_disk,
        &observed.manifest,
        mode,
    ));
    steps
}

/// Plans only the two indexes, as a sync would: for when the list of repos changed (a clone) but
/// the skills didn't.
#[must_use]
pub fn index_plan(observed: &Observed, repos: &[RepoContext]) -> Vec<Step> {
    plan(
        indexes(&observed.skills, repos),
        &observed.on_disk,
        &observed.manifest,
        Mode::Sync,
    )
}

fn indexes(skills: &[SkillInfo], repos: &[RepoContext]) -> Vec<Wanted> {
    vec![
        Wanted {
            path: CONTEXT_INDEX.to_owned(),
            contents: context_index(skills, repos),
        },
        Wanted {
            path: ROOT_INDEX.to_owned(),
            contents: root_index(),
        },
    ]
}

/// Gives a cloned repo its context folder, `context/projects/<dir>/`, with a starter README, if the folder
/// doesn't exist yet. Returns what it created. An existing folder is the person's: it is never
/// changed, even if it has no README.
///
/// # Errors
///
/// I/O errors.
pub fn ensure_repo_context(paths: &Paths, repo: &RepoContext) -> Result<Vec<PathBuf>, StoreError> {
    let dir = paths.root.join("context/projects").join(&repo.dir);
    if dir.symlink_metadata().is_ok() {
        return Ok(Vec::new());
    }
    std::fs::create_dir_all(&dir).map_err(|source| io(&dir, source))?;
    let readme = dir.join("README.md");
    let text = format!(
        "# {}\n\nNotes, plans and context for this repo, for people and every agent working on it.\n\
         Anything goes: mori wrote this file once and won't change it.\n",
        repo.repo
    );
    replace(&readme, &text)?;
    Ok(vec![dir, readme])
}

/// Moves a root from the old layout (`skills/` and `skills/llms.txt` at the top) to `context/`:
/// files mori wrote there and nobody changed are removed, so the plan reinstalls them under
/// `context/`; edited ones stay where they are and are returned, so the person can move them.
/// Anything mori never wrote is left alone. Empty old directories are removed.
///
/// # Errors
///
/// I/O errors.
pub fn migrate_legacy(paths: &Paths) -> Result<Vec<PathBuf>, StoreError> {
    let observed = observe(paths)?;
    let legacy: Vec<String> = observed
        .manifest
        .keys()
        .filter(|path| path.starts_with("skills/"))
        .cloned()
        .collect();
    if legacy.is_empty() {
        return Ok(Vec::new());
    }
    let mut manifest = observed.manifest;
    let mut kept = Vec::new();
    for relative in legacy {
        let path = paths.root.join(&relative);
        let recorded = manifest.remove(&relative);
        match file_hash(&path)? {
            Some(now) if Some(&now) == recorded.as_ref() => {
                std::fs::remove_file(&path).map_err(|source| io(&path, source))?;
            }
            Some(_) => kept.push(path),
            None => {}
        }
    }
    // Best effort: an old directory that still holds something isn't empty, and stays.
    let old = paths.root.join("skills");
    if let Ok(entries) = std::fs::read_dir(&old) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_dir(entry.path());
        }
    }
    let _ = std::fs::remove_dir(&old);
    write_manifest(paths, manifest)?;
    Ok(kept)
}

/// Writes the files the plan writes, each all at once, then records them in the manifest.
/// Returns the paths written. A file's hash goes into the manifest only after the file is in
/// place, so an interrupted run leaves files the next run treats as edited, never as mori's.
///
/// # Errors
///
/// I/O errors. Files written before the error stay written and recorded.
pub fn apply(
    paths: &Paths,
    steps: &[Step],
    mut manifest: Manifest,
) -> Result<Vec<PathBuf>, StoreError> {
    let mut written = Vec::new();
    let result = steps
        .iter()
        .filter(|step| step.action.writes())
        .try_for_each(|step| {
            let path = paths.root.join(&step.wanted.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|source| io(parent, source))?;
            }
            replace(&path, &step.wanted.contents)?;
            manifest.insert(step.wanted.path.clone(), hash(&step.wanted.contents));
            written.push(path);
            Ok(())
        });
    if !written.is_empty() {
        write_manifest(paths, manifest)?;
    }
    result.map(|()| written)
}

fn write_manifest(paths: &Paths, manifest: Manifest) -> Result<(), StoreError> {
    let file = ManifestFile {
        version: THIS_VERSION.to_owned(),
        files: manifest,
    };
    let text = serde_json::to_string_pretty(&file).map_err(|error| StoreError::Io {
        path: paths.state_dir.join(MANIFEST),
        source: std::io::Error::other(error),
    })?;
    replace(&paths.state_dir.join(MANIFEST), &(text + "\n"))
}

/// Writes `contents` to a temporary file beside `path`, then renames it over `path`, so readers
/// see the old file or the new one, never a partial one.
fn replace(path: &Path, contents: &str) -> Result<(), StoreError> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".tmp-{}", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(FILE_MODE)
        .open(&temporary)
        .and_then(|mut file| {
            file.write_all(contents.as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&temporary, path));
    if written.is_err() {
        // Best effort, and safe: this call created the temporary file.
        let _ = std::fs::remove_file(&temporary);
    }
    written.map_err(|source| io(path, source))
}

fn io(path: &Path, source: std::io::Error) -> StoreError {
    StoreError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use mori_core::paths::Env;
    use mori_core::skills::Action;

    use super::*;

    fn paths(home: &Path) -> Paths {
        Paths::resolve(&Env {
            home: Some(home.to_path_buf()),
            ..Env::default()
        })
        .unwrap()
    }

    fn sync(paths: &Paths, mode: Mode) -> Vec<Step> {
        let observed = observe(paths).unwrap();
        let steps = sync_plan(&observed, mode, &[]);
        std::fs::create_dir_all(&paths.state_dir).unwrap();
        apply(paths, &steps, observed.manifest).unwrap();
        steps
    }

    fn action(steps: &[Step], path: &str) -> Action {
        steps
            .iter()
            .find(|step| step.wanted.path == path)
            .map(|step| step.action)
            .unwrap()
    }

    #[test]
    fn every_skill_the_repo_indexes_is_embedded() {
        let index = include_str!("../../../skills/llms.txt");
        let listed: BTreeSet<&str> = index
            .lines()
            .filter_map(|line| line.strip_prefix("- ["))
            .filter_map(|line| line.split(']').next())
            .collect();
        let embedded: BTreeSet<&str> = EMBEDDED.iter().map(|(dir, _)| *dir).collect();

        assert_eq!(listed, embedded);
    }

    #[test]
    fn a_first_sync_installs_everything() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());

        let steps = sync(&paths, Mode::AddOnly);

        assert!(steps.iter().all(|step| step.action == Action::Install));
        assert!(
            paths
                .root
                .join("context/skills/using-mori/SKILL.md")
                .is_file()
        );
        let index = std::fs::read_to_string(paths.root.join(CONTEXT_INDEX)).unwrap();
        assert!(index.contains("- [lead-tree](skills/lead-tree/SKILL.md): "));
        assert!(index.contains("- [using-mori](skills/using-mori/SKILL.md): "));
        assert!(paths.root.join(ROOT_INDEX).is_file());
        assert!(paths.state_dir.join(MANIFEST).is_file());
    }

    #[test]
    fn a_second_sync_changes_nothing() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        sync(&paths, Mode::Sync);

        let steps = sync(&paths, Mode::Sync);

        assert!(steps.iter().all(|step| step.action == Action::Unchanged));
    }

    #[test]
    fn an_old_untouched_skill_is_updated_and_an_edited_one_kept() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        sync(&paths, Mode::Sync);
        // Pretend an older mori wrote different contents for both skills, then someone edited
        // one of them.
        let old = "---\nname: old\n---\n";
        let mut manifest = observe(&paths).unwrap().manifest;
        for dir in ["lead-tree", "using-mori"] {
            std::fs::write(paths.root.join(skill_path(dir)), old).unwrap();
            manifest.insert(skill_path(dir), hash(old));
        }
        let file = ManifestFile {
            version: "0.0.0".to_owned(),
            files: manifest,
        };
        std::fs::write(
            paths.state_dir.join(MANIFEST),
            serde_json::to_string(&file).unwrap(),
        )
        .unwrap();
        std::fs::write(paths.root.join(skill_path("using-mori")), "my edit\n").unwrap();

        let steps = sync(&paths, Mode::Sync);

        assert_eq!(action(&steps, &skill_path("lead-tree")), Action::Update);
        assert_eq!(
            action(&steps, &skill_path("using-mori")),
            Action::KeptEdited
        );
        assert_eq!(
            std::fs::read_to_string(paths.root.join(skill_path("using-mori"))).unwrap(),
            "my edit\n"
        );
    }

    #[test]
    fn a_persons_own_skill_is_indexed_and_left_alone() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        let mine = paths.root.join("context/skills/my-flow/SKILL.md");
        std::fs::create_dir_all(mine.parent().unwrap()).unwrap();
        std::fs::write(&mine, "---\nname: my-flow\ndescription: Mine.\n---\n").unwrap();

        sync(&paths, Mode::Sync);

        let index = std::fs::read_to_string(paths.root.join(CONTEXT_INDEX)).unwrap();
        assert!(index.contains("- [my-flow](skills/my-flow/SKILL.md): Mine."));
        assert_eq!(
            std::fs::read_to_string(&mine).unwrap(),
            "---\nname: my-flow\ndescription: Mine.\n---\n"
        );
    }

    #[test]
    fn a_skill_of_the_same_name_that_mori_never_wrote_is_skipped() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        let theirs = paths.root.join(skill_path("using-mori"));
        std::fs::create_dir_all(theirs.parent().unwrap()).unwrap();
        std::fs::write(&theirs, "theirs\n").unwrap();

        let steps = sync(&paths, Mode::Sync);

        assert_eq!(
            action(&steps, &skill_path("using-mori")),
            Action::SkippedNotOurs
        );
        assert_eq!(std::fs::read_to_string(&theirs).unwrap(), "theirs\n");
    }

    #[test]
    fn an_old_root_moves_to_context() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        std::fs::create_dir_all(&paths.state_dir).unwrap();
        // An old root: mori wrote two files under skills/, and someone edited one of them.
        let mut manifest = Manifest::new();
        for (relative, contents) in [
            ("skills/lead-tree/SKILL.md", "old lead\n"),
            ("skills/using-mori/SKILL.md", "old using\n"),
        ] {
            let path = paths.root.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, contents).unwrap();
            manifest.insert(relative.to_owned(), hash(contents));
        }
        write_manifest(&paths, manifest).unwrap();
        let edited = paths.root.join("skills/using-mori/SKILL.md");
        std::fs::write(&edited, "my edit\n").unwrap();

        let kept = migrate_legacy(&paths).unwrap();

        assert_eq!(kept, std::slice::from_ref(&edited));
        assert!(!paths.root.join("skills/lead-tree").exists());
        assert_eq!(std::fs::read_to_string(&edited).unwrap(), "my edit\n");
        assert!(
            observe(&paths)
                .unwrap()
                .manifest
                .keys()
                .all(|path| !path.starts_with("skills/"))
        );
        sync(&paths, Mode::Sync);
        assert!(
            paths
                .root
                .join("context/skills/lead-tree/SKILL.md")
                .is_file()
        );
    }

    #[test]
    fn a_repo_gets_a_context_folder_once() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths(home.path());
        let widget = RepoContext {
            dir: "widget".to_owned(),
            repo: "github.com/acme/widget".to_owned(),
        };

        let created = ensure_repo_context(&paths, &widget).unwrap();

        let readme = paths.root.join("context/projects/widget/README.md");
        assert_eq!(
            created,
            [paths.root.join("context/projects/widget"), readme.clone()]
        );
        assert!(
            std::fs::read_to_string(&readme)
                .unwrap()
                .starts_with("# github.com/acme/widget\n")
        );
        std::fs::write(&readme, "mine\n").unwrap();
        assert_eq!(
            ensure_repo_context(&paths, &widget).unwrap(),
            Vec::<PathBuf>::new()
        );
        assert_eq!(std::fs::read_to_string(&readme).unwrap(), "mine\n");
    }
}
