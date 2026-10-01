//! The filesystem side of disk usage: how much a directory holds, and how much the disk has free.
//! `mori-core::disk` decides what the numbers mean.

use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Condvar, Mutex, PoisonError};

use mori_core::disk::{OutputBase, Space};

use crate::StoreError;

/// What a directory holds on disk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Measured {
    /// Bytes allocated: blocks, not apparent sizes, with each hard-linked file counted once.
    pub bytes: u64,
    /// True if some entries couldn't be read, so `bytes` is short of the truth.
    pub partial: bool,
}

/// What the workers share while walking one tree.
struct Walk {
    /// Directories still to read, and how many workers are reading one now.
    queue: Mutex<(Vec<PathBuf>, usize)>,
    ready: Condvar,
    /// `(device, inode)` of files with more than one link, so each counts once.
    seen: Mutex<HashSet<(u64, u64)>>,
    device: u64,
}

/// Adds up everything under `root` on its filesystem, without following symlinks, on as many
/// threads as the machine has cores. A missing `root` holds nothing.
#[must_use]
pub fn measure(root: &Path) -> Measured {
    let Ok(meta) = root.symlink_metadata() else {
        return Measured::default();
    };
    if !meta.is_dir() {
        return Measured {
            bytes: allocated(&meta),
            partial: false,
        };
    }
    let walk = Walk {
        queue: Mutex::new((vec![root.to_path_buf()], 0)),
        ready: Condvar::new(),
        seen: Mutex::new(HashSet::new()),
        device: meta.dev(),
    };
    let threads = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let totals: Vec<Measured> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| scope.spawn(|| worker(&walk)))
            .collect();
        workers
            .into_iter()
            .map(|worker| {
                worker.join().unwrap_or(Measured {
                    bytes: 0,
                    partial: true,
                })
            })
            .collect()
    });
    totals.into_iter().fold(
        Measured {
            bytes: allocated(&meta),
            partial: false,
        },
        |sum, part| Measured {
            bytes: sum.bytes.saturating_add(part.bytes),
            partial: sum.partial || part.partial,
        },
    )
}

/// Takes directories off the queue until none are left and no other worker can add more.
fn worker(walk: &Walk) -> Measured {
    let mut total = Measured::default();
    loop {
        let dir = {
            let mut queue = walk.queue.lock().unwrap_or_else(PoisonError::into_inner);
            loop {
                if let Some(dir) = queue.0.pop() {
                    queue.1 += 1;
                    break Some(dir);
                }
                if queue.1 == 0 {
                    break None;
                }
                queue = walk
                    .ready
                    .wait(queue)
                    .unwrap_or_else(PoisonError::into_inner);
            }
        };
        let Some(dir) = dir else {
            walk.ready.notify_all();
            return total;
        };
        let mut found = Vec::new();
        read_dir(walk, &dir, &mut total, &mut found);
        let mut queue = walk.queue.lock().unwrap_or_else(PoisonError::into_inner);
        queue.0.extend(found);
        queue.1 -= 1;
        walk.ready.notify_all();
    }
}

/// Counts the entries of `dir`, and puts its subdirectories on the same filesystem in `found`.
fn read_dir(walk: &Walk, dir: &Path, total: &mut Measured, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        total.partial = true;
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            total.partial = true;
            continue;
        };
        let Ok(meta) = entry.path().symlink_metadata() else {
            total.partial = true;
            continue;
        };
        if meta.dev() != walk.device {
            continue;
        }
        if meta.is_dir() {
            found.push(entry.path());
        } else if meta.nlink() > 1
            && !walk
                .seen
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert((meta.dev(), meta.ino()))
        {
            continue;
        }
        total.bytes = total.bytes.saturating_add(allocated(&meta));
    }
}

/// The bytes an entry occupies: its 512-byte blocks.
fn allocated(meta: &std::fs::Metadata) -> u64 {
    meta.blocks().saturating_mul(512)
}

/// Free and total space on the disk holding `path`, as `df -Pk` reports them. `df` keeps mori
/// free of unsafe code, and works the same on Linux and macOS.
///
/// # Errors
///
/// [`StoreError::Io`] if `df` can't run, fails, or prints something unexpected.
pub fn free_space(path: &Path) -> Result<Space, StoreError> {
    let io = |source: std::io::Error| StoreError::Io {
        path: path.to_path_buf(),
        source,
    };
    let output = Command::new("df")
        .arg("-Pk")
        .arg(path)
        .env("LC_ALL", "C")
        .output()
        .map_err(io)?;
    let unexpected = || io(std::io::Error::other("unexpected output from `df -Pk`"));
    if !output.status.success() {
        return Err(unexpected());
    }
    parse_df(&String::from_utf8_lossy(&output.stdout)).ok_or_else(unexpected)
}

/// Reads `df -Pk` output: a header, then one line whose second and fourth columns are total and
/// available 1024-byte blocks. A long device name may wrap the line, so columns are counted from
/// the end.
fn parse_df(stdout: &str) -> Option<Space> {
    let fields: Vec<&str> = stdout
        .lines()
        .skip(1)
        .flat_map(str::split_whitespace)
        .collect();
    // Filesystem, 1024-blocks, Used, Available, Capacity, Mounted on (which may hold spaces, so
    // find the capacity column, the one ending in %).
    let capacity = fields.iter().rposition(|field| field.ends_with('%'))?;
    let available: u64 = fields.get(capacity.checked_sub(1)?)?.parse().ok()?;
    let total: u64 = fields.get(capacity.checked_sub(3)?)?.parse().ok()?;
    Some(Space {
        free: available.saturating_mul(1024),
        total: total.saturating_mul(1024),
    })
}

/// Every Bazel output base directly under `user_root`: the directories with a
/// `DO_NOT_BUILD_HERE` file. `install/`, `cache/` and anything else are skipped. A missing root
/// has none.
#[must_use]
pub fn output_bases(user_root: &Path) -> Vec<OutputBase> {
    let Ok(entries) = std::fs::read_dir(user_root) else {
        return Vec::new();
    };
    let mut bases: Vec<OutputBase> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            let path = entry.path();
            let named = std::fs::read_to_string(path.join("DO_NOT_BUILD_HERE")).ok()?;
            let workspace = PathBuf::from(named.trim());
            Some(OutputBase {
                workspace_exists: workspace.symlink_metadata().is_ok(),
                server_running: server_running(&path),
                workspace,
                path,
            })
        })
        .collect();
    bases.sort_by(|a, b| a.path.cmp(&b.path));
    bases
}

/// Whether a Bazel server may still run on `base`: its pid file names a live process, or can't
/// be read. Unsure counts as running, so mori never deletes under a live server.
fn server_running(base: &Path) -> bool {
    let pid_file = base.join("server/server.pid.txt");
    match std::fs::read_to_string(&pid_file) {
        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
        Ok(text) => match text.trim().parse::<u32>() {
            Err(_) => true,
            Ok(pid) => Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .output()
                .map_or(true, |output| output.status.success()),
        },
    }
}

/// Deletes an output base. Bazel makes parts of it read-only, so everything in it is made
/// writable by its owner first. Symlinks are deleted, never followed.
///
/// # Errors
///
/// [`StoreError::Io`] if it can't be deleted.
pub fn remove_output_base(base: &Path) -> Result<(), StoreError> {
    make_writable(base);
    match std::fs::remove_dir_all(base) {
        Err(source) if source.kind() != std::io::ErrorKind::NotFound => Err(StoreError::Io {
            path: base.to_path_buf(),
            source,
        }),
        _ => Ok(()),
    }
}

fn make_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(meta) = path.symlink_metadata() else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    let mode = meta.permissions().mode();
    if mode & 0o200 == 0 || (meta.is_dir() && mode & 0o700 != 0o700) {
        let wanted = if meta.is_dir() {
            mode | 0o700
        } else {
            mode | 0o200
        };
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(wanted));
    }
    if meta.is_dir()
        && let Ok(entries) = std::fs::read_dir(path)
    {
        for entry in entries.filter_map(Result::ok) {
            make_writable(&entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tree_counts_its_files_and_its_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
        std::fs::write(dir.path().join("a/b/c/big"), vec![1_u8; 256 * 1024]).unwrap();
        std::fs::write(dir.path().join("a/small"), b"hi").unwrap();

        let measured = measure(dir.path());

        assert!(measured.bytes >= 256 * 1024, "{measured:?}");
        assert!(!measured.partial);
    }

    #[test]
    fn a_hard_linked_file_counts_once() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("one"), vec![1_u8; 256 * 1024]).unwrap();
        let single = measure(dir.path()).bytes;
        std::fs::hard_link(dir.path().join("one"), dir.path().join("two")).unwrap();

        assert_eq!(measure(dir.path()).bytes, single);
    }

    #[test]
    fn a_symlink_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        std::fs::write(elsewhere.path().join("big"), vec![1_u8; 512 * 1024]).unwrap();
        std::os::unix::fs::symlink(elsewhere.path(), dir.path().join("link")).unwrap();

        assert!(measure(dir.path()).bytes < 512 * 1024);
    }

    #[test]
    fn a_missing_directory_holds_nothing() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(measure(&dir.path().join("gone")), Measured::default());
    }

    #[test]
    fn df_output_is_read_from_the_capacity_column() {
        let linux = "Filesystem     1024-blocks      Used Available Capacity Mounted on\n\
                     /dev/nvme0n1p2   3907018584 3600000000 307018584      93% /\n";
        let wrapped = "Filesystem 1024-blocks Used Available Capacity Mounted on\n\
                       /dev/mapper/a-very-long-volume-name\n\
                       \x20     100 40 60 40% /Volumes/My Disk\n";

        assert_eq!(
            parse_df(linux),
            Some(Space {
                free: 307_018_584 * 1024,
                total: 3_907_018_584 * 1024,
            })
        );
        assert_eq!(
            parse_df(wrapped),
            Some(Space {
                free: 60 * 1024,
                total: 100 * 1024,
            })
        );
        assert_eq!(parse_df("Filesystem\n"), None);
    }

    fn fake_base(root: &Path, hash: &str, workspace: &Path) -> PathBuf {
        let base = root.join(hash);
        std::fs::create_dir_all(base.join("execroot/_main")).unwrap();
        std::fs::write(
            base.join("DO_NOT_BUILD_HERE"),
            format!("{}\n", workspace.display()),
        )
        .unwrap();
        base
    }

    #[test]
    fn output_bases_are_the_directories_naming_a_workspace() {
        let root = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        fake_base(root.path(), "aaa", workspace.path());
        fake_base(root.path(), "bbb", &workspace.path().join("gone"));
        std::fs::create_dir_all(root.path().join("install/1234")).unwrap();

        let bases = output_bases(root.path());

        assert_eq!(bases.len(), 2, "{bases:?}");
        assert_eq!(bases[0].workspace, workspace.path());
        assert!(bases[0].workspace_exists);
        assert!(!bases[1].workspace_exists);
        assert!(!bases[1].server_running);
        assert_eq!(output_bases(&root.path().join("missing")), []);
    }

    #[test]
    fn a_live_or_unreadable_server_pid_counts_as_running() {
        let root = tempfile::tempdir().unwrap();
        let base = fake_base(root.path(), "aaa", Path::new("/nowhere"));
        std::fs::create_dir_all(base.join("server")).unwrap();

        std::fs::write(
            base.join("server/server.pid.txt"),
            std::process::id().to_string(),
        )
        .unwrap();
        assert!(server_running(&base));
        std::fs::write(base.join("server/server.pid.txt"), "not a pid").unwrap();
        assert!(server_running(&base));
        std::fs::remove_file(base.join("server/server.pid.txt")).unwrap();
        assert!(!server_running(&base));
    }

    #[test]
    fn a_read_only_output_base_is_removed() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("keep"), b"keep").unwrap();
        let base = fake_base(root.path(), "aaa", Path::new("/nowhere"));
        let sealed = base.join("execroot/_main/external");
        std::fs::create_dir_all(&sealed).unwrap();
        std::fs::write(sealed.join("file"), b"x").unwrap();
        std::os::unix::fs::symlink(outside.path(), base.join("link")).unwrap();
        std::fs::set_permissions(sealed.join("file"), std::fs::Permissions::from_mode(0o444))
            .unwrap();
        std::fs::set_permissions(&sealed, std::fs::Permissions::from_mode(0o555)).unwrap();

        remove_output_base(&base).unwrap();

        assert!(!base.exists());
        assert!(outside.path().join("keep").exists());
    }

    #[test]
    fn free_space_reads_a_real_disk() {
        let dir = tempfile::tempdir().unwrap();

        let space = free_space(dir.path()).unwrap();

        assert!(space.total > 0 && space.free <= space.total, "{space:?}");
    }
}
