//! How text looks on a terminal: colour and glyphs for people, plain text everywhere else.
//!
//! Decided once at startup (`docs/cli-style.md`): colour only when the stream is a terminal,
//! `NO_COLOR` is unset, `TERM` isn't `dumb` and `--json` isn't given (`--color` overrides);
//! glyphs only on a terminal with a UTF-8 locale. `--json` is never touched. A colour or glyph
//! never carries meaning that a word next to it doesn't.

use std::io::IsTerminal;
use std::sync::OnceLock;

use anstyle::{AnsiColor, Effects, Style};

/// When to use colour (`--color`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorWhen {
    /// On a terminal, unless `NO_COLOR` is set or `TERM` is `dumb`.
    #[default]
    Auto,
    /// Always, even into a pipe.
    Always,
    /// Never.
    Never,
}

/// What a piece of text is, which decides its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Clean, created, removed, landed.
    Ok,
    /// Edited, unpushed, warnings.
    Warn,
    /// Refusals and problems.
    Problem,
    /// Paths, IDs and other metadata.
    Quiet,
    /// The names of repos and trees.
    Name,
}

#[derive(Clone, Copy, Debug, Default)]
struct Look {
    stdout_color: bool,
    stderr_color: bool,
    glyphs: bool,
}

static LOOK: OnceLock<Look> = OnceLock::new();

/// Decides the look for this run. Before it is called, everything is plain.
pub fn init(when: ColorWhen, json: bool) {
    let env_allows = std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
        && std::env::var("TERM").map_or(true, |term| term != "dumb");
    let color = |terminal: bool| {
        !json
            && match when {
                ColorWhen::Always => true,
                ColorWhen::Never => false,
                ColorWhen::Auto => terminal && env_allows,
            }
    };
    let _ = LOOK.set(Look {
        stdout_color: color(std::io::stdout().is_terminal()),
        stderr_color: color(std::io::stderr().is_terminal()),
        glyphs: !json && std::io::stdout().is_terminal() && utf8_locale(),
    });
}

fn look() -> Look {
    LOOK.get().copied().unwrap_or_default()
}

/// Whether the locale says the terminal can show UTF-8.
fn utf8_locale() -> bool {
    ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .is_some_and(|locale| {
            let locale = locale.to_ascii_lowercase();
            locale.contains("utf-8") || locale.contains("utf8")
        })
}

fn style(role: Role) -> Style {
    match role {
        Role::Ok => AnsiColor::Green.on_default(),
        Role::Warn => AnsiColor::Yellow.on_default(),
        Role::Problem => AnsiColor::Red.on_default().effects(Effects::BOLD),
        Role::Quiet => AnsiColor::BrightBlack.on_default(),
        Role::Name => Style::new().effects(Effects::BOLD),
    }
}

fn paint_if(on: bool, role: Role, text: &str) -> String {
    if on && !text.is_empty() {
        let style = style(role);
        format!("{style}{text}{style:#}")
    } else {
        text.to_owned()
    }
}

/// `text` in `role`'s colour, for stdout.
pub fn paint(role: Role, text: &str) -> String {
    paint_if(look().stdout_color, role, text)
}

/// `text` in `role`'s colour, for stderr.
pub fn paint_err(role: Role, text: &str) -> String {
    paint_if(look().stderr_color, role, text)
}

/// Whether box drawing and state marks can be shown.
pub fn glyphs() -> bool {
    look().glyphs
}

/// The state mark for `role`: `✓ ! ✗`, or `ok ! x` without glyphs.
pub fn mark(role: Role) -> &'static str {
    match (role, glyphs()) {
        (Role::Ok, true) => "✓",
        (Role::Ok, false) => "ok",
        (Role::Problem, true) => "✗",
        (Role::Problem, false) => "x",
        _ => "!",
    }
}

/// The branch in front of a child in a tree view: `├─`/`└─`, or `|-`/`` `- `` without glyphs.
pub fn branch(last: bool) -> &'static str {
    match (last, glyphs()) {
        (false, true) => "├─",
        (true, true) => "└─",
        (false, false) => "|-",
        (true, false) => "`-",
    }
}

/// What continues a parent's line past a child: `│ `, or `| ` without glyphs.
pub fn trunk(last: bool) -> &'static str {
    match (last, glyphs()) {
        (true, _) => "  ",
        (false, true) => "│ ",
        (false, false) => "| ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_until_decided() {
        assert_eq!(paint(Role::Ok, "clean"), "clean");
        assert_eq!(mark(Role::Problem), "x");
        assert_eq!(branch(true), "`-");
    }

    #[test]
    fn colour_wraps_and_resets() {
        let painted = paint_if(true, Role::Warn, "edited");

        assert!(painted.starts_with("\u{1b}["), "{painted:?}");
        assert!(painted.contains("edited"));
        assert!(painted.ends_with("\u{1b}[0m"), "{painted:?}");
    }
}
