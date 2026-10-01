//! Disk usage: sizes, the free-space floor, and the `[disk]` policy.
//!
//! mori sets no limits by default. It measures, warns when the disk is nearly full, and leaves
//! the decision to the person; see `docs/design/core/disk-usage.md`.

use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

/// `[disk]` in `config.toml`. Every key has a default.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiskPolicy {
    /// Warn when free space on the disk under the root drops below this.
    pub warn_below: Floor,
}

/// How little free space is too little: a share of the disk, or a size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum Floor {
    /// A percentage of the disk's size, 0 to 100.
    Percent(u8),
    /// A number of bytes.
    Bytes(u64),
}

impl Default for Floor {
    fn default() -> Self {
        Self::Percent(10)
    }
}

impl FromStr for Floor {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        if let Some(percent) = text.strip_suffix('%') {
            return match percent.trim().parse::<u8>() {
                Ok(percent) if percent <= 100 => Ok(Self::Percent(percent)),
                _ => Err(format!(
                    "expected a percentage from 0% to 100%, got \"{text}\""
                )),
            };
        }
        parse_size(text).map(Self::Bytes)
    }
}

impl TryFrom<String> for Floor {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl fmt::Display for Floor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percent(percent) => write!(f, "{percent}%"),
            Self::Bytes(bytes) => f.write_str(&format_size(*bytes)),
        }
    }
}

/// Free and total space on one disk, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Space {
    /// Space an unprivileged user can still write.
    pub free: u64,
    /// The disk's size.
    pub total: u64,
}

impl Space {
    /// Free space as a whole percentage of the disk, rounded down.
    #[must_use]
    pub fn free_percent(self) -> u64 {
        if self.total == 0 {
            return 0;
        }
        u64::try_from(u128::from(self.free) * 100 / u128::from(self.total)).unwrap_or(100)
    }

    /// Whether free space is below `floor`.
    #[must_use]
    pub fn below(self, floor: Floor) -> bool {
        match floor {
            Floor::Bytes(bytes) => self.free < bytes,
            Floor::Percent(percent) => {
                u128::from(self.free) * 100 < u128::from(self.total) * u128::from(percent)
            }
        }
    }
}

/// The warning for free space below `floor`, if it is.
#[must_use]
pub fn low_space_warning(space: Space, floor: Floor) -> Option<String> {
    space.below(floor).then(|| {
        format!(
            "only {} free of {} ({}%), below the {floor} floor; `mori gc` lists the trees that \
             are safe to remove, and shared caches keep trees small: \
             https://nevzheng.github.io/mori/shared-caches/",
            format_size(space.free),
            format_size(space.total),
            space.free_percent(),
        )
    })
}

/// Something `gc --free` could remove, as far as picking goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// What removing it frees.
    pub bytes: u64,
    /// Seconds since its latest change; none when unknown.
    pub idle_seconds: Option<u64>,
    /// True if it holds only cache (Bazel output left by a deleted tree), so it goes first.
    pub cache_only: bool,
}

/// Which candidates `gc --free` removes to free `target` bytes, as indexes into `candidates`:
/// cache first, then the least recently changed (unknown change times last), until the total
/// reaches the target. The last pick may overshoot. Candidates that free nothing are never picked.
/// If everything together frees less than the target, everything is picked.
#[must_use]
pub fn pick_to_free(candidates: &[Candidate], target: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..candidates.len())
        .filter(|&index| candidates[index].bytes > 0)
        .collect();
    order.sort_by_key(|&index| {
        let candidate = candidates[index];
        (
            !candidate.cache_only,
            candidate.idle_seconds.is_none(),
            std::cmp::Reverse(candidate.idle_seconds.unwrap_or(0)),
        )
    });
    let mut freed = 0_u64;
    let mut picked = Vec::new();
    for index in order {
        if freed >= target {
            break;
        }
        freed = freed.saturating_add(candidates[index].bytes);
        picked.push(index);
    }
    picked
}

/// How long a measured size is reused before `ls --size` or `gc` measures again.
pub const SIZE_REUSE_SECONDS: u64 = 15 * 60;

/// Whether a size measured at `measured_at` can still be used at `now`.
#[must_use]
pub fn size_is_recent(measured_at: u64, now: u64) -> bool {
    measured_at <= now && now - measured_at < SIZE_REUSE_SECONDS
}

/// Seconds since the Unix epoch as RFC 3339 in UTC, e.g. `2026-10-01T11:23:27Z`.
#[must_use]
pub fn rfc3339(seconds: u64) -> String {
    let days = seconds / 86_400;
    let rest = seconds % 86_400;
    // Howard Hinnant's civil_from_days, for days since 1970-01-01.
    let z = days + 719_468;
    let era = z / 146_097;
    let day_of_era = z % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

const UNITS: [(char, u32); 5] = [('K', 1), ('M', 2), ('G', 3), ('T', 4), ('P', 5)];

/// Parses a size such as `200G`, `1.5T`, `512M` or `4096`. Units are powers of 1024, with an
/// optional `B` or `iB` after the letter.
///
/// # Errors
///
/// A message naming the text, if it isn't a size.
pub fn parse_size(text: &str) -> Result<u64, String> {
    let invalid = || format!("expected a size such as 200G or 1.5T, got \"{text}\"");
    let trimmed = text.trim();
    let upper = trimmed.to_ascii_uppercase();
    let unitless = upper
        .strip_suffix("IB")
        .or_else(|| upper.strip_suffix('B'))
        .unwrap_or(&upper);
    let (number, power) = match unitless.chars().last() {
        Some(letter) if letter.is_ascii_alphabetic() => {
            let power = UNITS
                .iter()
                .find(|(unit, _)| *unit == letter)
                .map(|(_, power)| *power)
                .ok_or_else(invalid)?;
            (&unitless[..unitless.len() - 1], power)
        }
        _ => (unitless, 0),
    };
    let number = number.trim();
    if number.is_empty() || number.starts_with('-') || number.starts_with('+') {
        return Err(invalid());
    }
    let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
    if !whole.chars().all(|c| c.is_ascii_digit())
        || !fraction.chars().all(|c| c.is_ascii_digit())
        || (whole.is_empty() && fraction.is_empty())
    {
        return Err(invalid());
    }
    let scale = 1024_u128.pow(power);
    let whole: u128 = if whole.is_empty() {
        0
    } else {
        whole.parse().map_err(|_| invalid())?
    };
    let mut bytes = whole.checked_mul(scale).ok_or_else(invalid)?;
    if !fraction.is_empty() {
        let digits = u32::try_from(fraction.len()).map_err(|_| invalid())?;
        let denominator = 10_u128.checked_pow(digits).ok_or_else(invalid)?;
        let numerator: u128 = fraction.parse().map_err(|_| invalid())?;
        bytes += numerator * scale / denominator;
    }
    u64::try_from(bytes).map_err(|_| invalid())
}

/// Formats a size for people: `512B`, `31M`, `1.9T`. One decimal below 10 of a unit.
#[must_use]
pub fn format_size(bytes: u64) -> String {
    let mut unit = None;
    for (letter, power) in UNITS.iter().rev() {
        if bytes >= 1024_u64.pow(*power) {
            unit = Some((*letter, 1024_u64.pow(*power)));
            break;
        }
    }
    let Some((letter, scale)) = unit else {
        return format!("{bytes}B");
    };
    let tenths = u128::from(bytes) * 10 / u128::from(scale);
    if tenths < 100 {
        format!("{}.{}{letter}", tenths / 10, tenths % 10)
    } else {
        format!("{}{letter}", tenths / 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: u64 = 1024 * 1024 * 1024;

    #[test]
    fn sizes_parse_in_powers_of_1024() {
        assert_eq!(parse_size("4096"), Ok(4096));
        assert_eq!(parse_size("1K"), Ok(1024));
        assert_eq!(parse_size("200G"), Ok(200 * G));
        assert_eq!(parse_size("200gb"), Ok(200 * G));
        assert_eq!(parse_size("200GiB"), Ok(200 * G));
        assert_eq!(parse_size("1.5T"), Ok(1536 * G));
        assert_eq!(parse_size(" 2 M "), Ok(2 * 1024 * 1024));
    }

    #[test]
    fn non_sizes_are_refused() {
        for text in ["", "G", "-1G", "1X", "1.2.3G", "lots", "99999999999P", "."] {
            assert!(parse_size(text).is_err(), "for {text:?}");
        }
    }

    #[test]
    fn sizes_format_short() {
        assert_eq!(format_size(0), "0B");
        assert_eq!(format_size(512), "512B");
        assert_eq!(format_size(1536), "1.5K");
        assert_eq!(format_size(31 * 1024 * 1024), "31M");
        assert_eq!(format_size(1946 * G), "1.9T");
        assert_eq!(format_size(412 * G), "412G");
    }

    #[test]
    fn floors_parse_as_a_share_or_a_size() {
        assert_eq!("10%".parse(), Ok(Floor::Percent(10)));
        assert_eq!("200G".parse(), Ok(Floor::Bytes(200 * G)));
        assert!("101%".parse::<Floor>().is_err());
        assert!("ten%".parse::<Floor>().is_err());
        assert_eq!(Floor::default(), Floor::Percent(10));
    }

    #[test]
    fn below_the_floor_warns() {
        let space = Space {
            free: 300 * G,
            total: 4000 * G,
        };

        assert!(space.below(Floor::Percent(10)));
        assert!(!space.below(Floor::Percent(5)));
        assert!(space.below(Floor::Bytes(400 * G)));
        assert!(!space.below(Floor::Bytes(100 * G)));
        assert_eq!(space.free_percent(), 7);
        let warning = low_space_warning(space, Floor::Percent(10)).unwrap();
        assert!(warning.contains("300G free of 3.9T (7%)"), "{warning}");
        assert!(warning.contains("`mori gc`"), "{warning}");
        assert_eq!(low_space_warning(space, Floor::Percent(5)), None);
    }

    fn candidate(bytes: u64, idle_seconds: Option<u64>) -> Candidate {
        Candidate {
            bytes,
            idle_seconds,
            cache_only: false,
        }
    }

    #[test]
    fn free_picks_the_least_recently_changed_first() {
        let candidates = [
            candidate(10 * G, Some(100)),
            candidate(10 * G, Some(900)),
            candidate(10 * G, None),
            candidate(10 * G, Some(500)),
        ];

        assert_eq!(pick_to_free(&candidates, 15 * G), [1, 3]);
        assert_eq!(pick_to_free(&candidates, 20 * G), [1, 3]);
        assert_eq!(pick_to_free(&candidates, 21 * G), [1, 3, 0]);
        assert_eq!(pick_to_free(&candidates, 100 * G), [1, 3, 0, 2]);
        assert_eq!(pick_to_free(&candidates, 0), Vec::<usize>::new());
    }

    #[test]
    fn free_takes_cache_first_and_skips_what_frees_nothing() {
        let candidates = [
            candidate(10 * G, Some(900)),
            candidate(0, Some(9999)),
            Candidate {
                bytes: 5 * G,
                idle_seconds: None,
                cache_only: true,
            },
        ];

        assert_eq!(pick_to_free(&candidates, 5 * G), [2]);
        assert_eq!(pick_to_free(&candidates, 6 * G), [2, 0]);
        assert_eq!(pick_to_free(&candidates, 100 * G), [2, 0]);
    }

    #[test]
    fn times_format_as_rfc3339() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339(1_790_853_807), "2026-10-01T11:23:27Z");
    }

    #[test]
    fn a_size_is_reused_for_fifteen_minutes() {
        assert!(size_is_recent(1000, 1000));
        assert!(size_is_recent(1000, 1000 + SIZE_REUSE_SECONDS - 1));
        assert!(!size_is_recent(1000, 1000 + SIZE_REUSE_SECONDS));
        assert!(
            !size_is_recent(2000, 1000),
            "a size from the future is not trusted"
        );
    }

    #[test]
    fn a_zero_floor_never_warns_and_100_percent_always_does() {
        let space = Space {
            free: G,
            total: 2 * G,
        };

        assert!(!space.below(Floor::Percent(0)));
        assert!(space.below(Floor::Percent(100)));
    }
}
