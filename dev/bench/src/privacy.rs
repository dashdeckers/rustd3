//! [`Identity`]: what a submitted archive must not hold (D22.9 (4)): the
//! home directory, the host name and the user name. Before a level packs
//! its results it rewrites every text file through [`Identity::scrub`]:
//! the results directory's own path goes (paths are relative to it), the
//! home directory becomes `~`, the host name `<host>` and the user name,
//! where it is a path component, `<user>`. What is written in the first
//! place is already limited (`system.txt` takes `uname -srm`, and only
//! whether the display variables are set); the scrub is the backstop for
//! paths in command lines, logs and panic messages. [`PRIVACY`] is the
//! archive's `PRIVACY.md`.

use std::fs;
use std::path::Path;
use std::process::Command;

/// The strings that identify the machine's owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The home directory (`HOME`, else `USERPROFILE`), written `~`.
    pub home: Option<String>,
    /// The host name, written `<host>`.
    pub host: Option<String>,
    /// The user name, written `<user>` where it is a path component.
    pub user: Option<String>,
}

/// `PRIVACY.md`: what an archive holds.
pub const PRIVACY: &str = "\
# What this archive holds

`rustd3-bench` wrote this archive for a performance report (D22.9). It
holds what the benchmark measured and a description of the machine, and
nothing else: no host name, no user name, no home path (paths are relative
to the results directory, or start with `~` for the home directory), no
file of yours, no screenshot, no network address, and environment
variables only from an allowlist. Every text file was rewritten before
packing to replace the home directory, the host name and the user name,
should any have appeared (in a panic message, say).

- `results.json` and `report.md`: the level's summary: the rustd3 version
  and commit, the level, the machine (operating system and its release,
  kernel, architecture, CPU model, cores, memory, GPU), the display (its
  refresh rate and scale factor, and how they were found), and per
  profile its cells' summary metrics and budgets.
- `system.txt`: the session type and desktop (`XDG_SESSION_TYPE`,
  `XDG_CURRENT_DESKTOP`), whether `WAYLAND_DISPLAY` and `DISPLAY` are set
  (not their values), what `xrandr --current`, `cosmic-randr list`,
  GNOME's scaling factors and `nvidia-smi` print about the displays and
  the GPU, `uname -srm` and `/etc/os-release`, each where present.
- `<profile>/meta.json`: the machine as above, the compiler, the rustd3
  and iced revisions, the run's settings and its command line.
- `<profile>/raw/*.jsonl`: per run, the benchmark's own timings: frames,
  the inputs it replayed, renders, and its process's CPU time and memory.
- `<profile>/replays/*.jsonl`: the generated input (no real input is
  recorded).
- `<profile>/*.csv` and `<profile>/report.md`: derived from the raw files.
- `run-<profile>.log`: the benchmark's console output.
";

impl Identity {
    /// This process's: `HOME` (or `USERPROFILE`), the host name
    /// (`HOSTNAME`, `COMPUTERNAME`, the kernel's, or `hostname`), and
    /// `USER` (or `USERNAME`, `LOGNAME`).
    pub fn here() -> Identity {
        let var = |names: &[&str]| {
            names
                .iter()
                .find_map(|name| std::env::var(name).ok())
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        };
        let host = var(&["HOSTNAME", "COMPUTERNAME"])
            .or_else(|| {
                fs::read_to_string("/proc/sys/kernel/hostname")
                    .ok()
                    .map(|name| name.trim().to_owned())
            })
            .or_else(|| {
                let out = Command::new("hostname").output().ok()?;
                out.status
                    .success()
                    .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
            })
            .filter(|name| !name.is_empty());
        Identity {
            home: var(&["HOME", "USERPROFILE"]),
            host,
            user: var(&["USER", "USERNAME", "LOGNAME"]),
        }
    }

    /// `text` with the results directory `root` (each spelling in
    /// `roots`: as given, absolute, canonical) made relative, then the
    /// home directory written `~`, the host name `<host>` and the user
    /// name, as a path component, `<user>`.
    pub fn scrub(&self, text: &str, roots: &[String]) -> String {
        let mut text = text.to_owned();
        // Each path as written, with `/` for `\`, and escaped as JSON
        // escapes a Windows path.
        let spellings = |path: &str| {
            let mut all = vec![path.to_owned()];
            if path.contains('\\') {
                all.extend([path.replace('\\', "/"), path.replace('\\', "\\\\")]);
            }
            all
        };
        let mut roots: Vec<String> = roots
            .iter()
            .filter(|r| r.len() > 1)
            .flat_map(|root| spellings(root))
            .collect();
        // The longest first: a relative spelling is a suffix of the others.
        roots.sort_by_key(|root| std::cmp::Reverse(root.len()));
        for root in roots {
            for separator in ["/", "\\\\", "\\"] {
                text = text.replace(&format!("{root}{separator}"), "");
            }
            text = replace_path(&text, &root, ".");
        }
        if let Some(home) = self.home.as_deref().filter(|home| home.len() > 1) {
            let mut homes = spellings(home);
            homes.sort_by_key(|home| std::cmp::Reverse(home.len()));
            for home in homes {
                text = replace_path(&text, &home, "~");
            }
        }
        if let Some(host) = self.host.as_deref().filter(|host| host.len() > 1) {
            // As written and in lower case (Windows prints its upper-case
            // name, DNS the lower-case one); not in every case, so that a
            // host called `vm` leaves "a VM" alone.
            text = replace_token(&text, host, "<host>");
            text = replace_token(&text, &host.to_lowercase(), "<host>");
        }
        if let Some(user) = self.user.as_deref().filter(|user| !user.is_empty()) {
            text = replace_component(&text, user, "<user>");
        }
        text
    }

    /// Rewrites every UTF-8 file under `dir` through [`Identity::scrub`],
    /// `dir` itself the results directory.
    pub fn scrub_dir(&self, dir: &Path) -> std::io::Result<()> {
        let mut roots = vec![dir.display().to_string()];
        if let Ok(absolute) = std::path::absolute(dir) {
            roots.push(absolute.display().to_string());
        }
        if let Ok(canonical) = dir.canonicalize() {
            roots.push(canonical.display().to_string());
        }
        let mut stack = vec![dir.to_path_buf()];
        while let Some(at) = stack.pop() {
            for entry in fs::read_dir(&at)? {
                let path = entry?.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Ok(text) = fs::read_to_string(&path) {
                    let clean = self.scrub(&text, &roots);
                    if clean != text {
                        fs::write(&path, clean)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Whether `c` continues a word of a path or a name.
fn word(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-')
}

/// `text` with every `from` that the character after does not continue
/// (a path ends there) replaced by `to`.
fn replace_path(text: &str, from: &str, to: &str) -> String {
    replace_where(text, from, to, false, |before, after| {
        before.is_none_or(|c| !word(c) && c != '.') && after.is_none_or(|c| !word(c))
    })
}

/// `text` with every whole token `from` replaced by `to`.
fn replace_token(text: &str, from: &str, to: &str) -> String {
    replace_where(text, from, to, false, |before, after| {
        before.is_none_or(|c| !word(c)) && after.is_none_or(|c| !word(c))
    })
}

/// `text` with every path component `from` (after `/` or `\`; ASCII case
/// ignored) replaced by `to`.
fn replace_component(text: &str, from: &str, to: &str) -> String {
    replace_where(text, from, to, true, |before, after| {
        matches!(before, Some('/' | '\\')) && after.is_none_or(|c| !word(c) && c != '.')
    })
}

/// `text` with each occurrence of `from` replaced by `to` where `keep`
/// accepts the characters around it; `fold` ignores ASCII case.
fn replace_where(
    text: &str,
    from: &str,
    to: &str,
    fold: bool,
    keep: impl Fn(Option<char>, Option<char>) -> bool,
) -> String {
    let (hay, needle) = if fold {
        (text.to_ascii_lowercase(), from.to_ascii_lowercase())
    } else {
        (text.to_owned(), from.to_owned())
    };
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut start = 0;
    while let Some(found) = hay[start..].find(&needle) {
        let at = start + found;
        let end = at + needle.len();
        let before = text[..at].chars().next_back();
        let after = text[end..].chars().next();
        if keep(before, after) {
            out.push_str(&text[last..at]);
            out.push_str(to);
            last = end;
        }
        start = end;
    }
    out.push_str(&text[last..]);
    out
}

#[cfg(test)]
mod tests {
    use super::{Identity, replace_component, replace_path, replace_token};

    #[test]
    fn paths_names_and_hosts_are_replaced_only_where_they_are_whole() {
        assert_eq!(
            replace_path("/root /rootfs /root/x", "/root", "~"),
            "~ /rootfs ~/x"
        );
        assert_eq!(
            replace_path("at /mnt/home/me.", "/home/me", "~"),
            "at /mnt/home/me."
        );
        assert_eq!(replace_path("in /home/me.", "/home/me", "~"), "in ~.");
        assert_eq!(
            replace_token("pop-os pop-os.local POP-OS xpop-os", "pop-os", "<host>"),
            "<host> <host>.local POP-OS xpop-os"
        );
        assert_eq!(
            replace_component(
                "/media/alice/disk C:\\Users\\Alice alice@ /alicex",
                "alice",
                "<user>"
            ),
            "/media/<user>/disk C:\\Users\\<user> alice@ /alicex"
        );
    }

    #[test]
    fn a_scrub_takes_the_results_directory_the_home_the_host_and_the_user_out() {
        let identity = Identity {
            home: Some("/home/alice".to_owned()),
            host: Some("ALICES-LAPTOP".to_owned()),
            user: Some("alice".to_owned()),
        };
        let roots = [
            "bench-results/20261029T120000Z-quick".to_owned(),
            "/home/alice/rustd3/bench-results/20261029T120000Z-quick".to_owned(),
        ];
        let text = "rustd3-bench run --out /home/alice/rustd3/bench-results/20261029T120000Z-quick/native\n\
                    report: bench-results/20261029T120000Z-quick/native/report.md\n\
                    thread 'main' panicked at /home/alice/.cargo/git/x.rs:1:2 on alices-laptop\n\
                    ALICES-LAPTOP\n\
                    /media/alice/disk";
        assert_eq!(
            identity.scrub(text, &roots),
            "rustd3-bench run --out native\n\
             report: native/report.md\n\
             thread 'main' panicked at ~/.cargo/git/x.rs:1:2 on <host>\n\
             <host>\n\
             /media/<user>/disk"
        );
        // A Windows home, as written and escaped in JSON.
        let windows = Identity {
            home: Some("C:\\Users\\Alice".to_owned()),
            host: Some("DESKTOP-7F3A".to_owned()),
            user: Some("Alice".to_owned()),
        };
        assert_eq!(
            windows.scrub(
                "C:\\Users\\Alice\\.cargo {\"out\": \"C:\\\\Users\\\\Alice\\\\x\"} desktop-7f3a",
                &[]
            ),
            "~\\.cargo {\"out\": \"~\\\\x\"} <host>"
        );
        // A home of "/" is no path to replace.
        let root = Identity {
            home: Some("/".to_owned()),
            host: None,
            user: None,
        };
        assert_eq!(root.scrub("/usr/bin", &[]), "/usr/bin");
    }
}
