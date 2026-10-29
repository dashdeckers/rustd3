//! The submission archive (D22.9 (4)): a results directory packed as a
//! ustar archive written here and gzipped with `flate2`, and read back in
//! memory by `compare` and `report`. Entries carry no owner (uid, gid and
//! their names are empty); their times are the pack's.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;

/// One file of an archive, read into memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Its path in the archive, `/`-separated.
    pub path: String,
    /// Its bytes.
    pub data: Vec<u8>,
}

/// A ustar block.
const BLOCK: usize = 512;

/// The record size archives are padded to, as tar writes them.
const RECORD: usize = 20 * BLOCK;

/// Packs `dir` into the gzipped archive `archive`, every entry under
/// `dir`'s own name, in a sorted order. It is written beside `archive` and
/// renamed over it, so an interrupted pack leaves the last whole archive.
/// Returns the archive's size in bytes.
pub fn pack(dir: &Path, archive: &Path) -> io::Result<u64> {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| invalid("a results directory without a name"))?;
    let mtime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut files = Vec::new();
    walk(dir, &name, &mut files)?;
    let partial = archive.with_extension("gz.partial");
    let mut out = GzEncoder::new(fs::File::create(&partial)?, Compression::default());
    let mut written = 0;
    out.write_all(&header(&format!("{name}/"), 0, b'5', mtime)?)?;
    written += BLOCK;
    for (path, source) in files {
        match source {
            None => {
                out.write_all(&header(&format!("{path}/"), 0, b'5', mtime)?)?;
                written += BLOCK;
            }
            Some(source) => {
                let data = fs::read(&source)?;
                out.write_all(&header(&path, data.len() as u64, b'0', mtime)?)?;
                out.write_all(&data)?;
                let padding = (BLOCK - data.len() % BLOCK) % BLOCK;
                out.write_all(&vec![0; padding])?;
                written += BLOCK + data.len() + padding;
            }
        }
    }
    // Two zero blocks end the archive; zeros pad it to a whole record.
    let end = 2 * BLOCK;
    let padding = (RECORD - (written + end) % RECORD) % RECORD;
    out.write_all(&vec![0; end + padding])?;
    out.finish()?.sync_all()?;
    fs::rename(&partial, archive)?;
    Ok(fs::metadata(archive)?.len())
}

/// The files of the gzipped tar archive at `path`, in its order:
/// ustar's names and prefixes, GNU long names and pax paths are read;
/// links and other kinds are skipped.
pub fn read(path: &Path) -> io::Result<Vec<Entry>> {
    let mut bytes = Vec::new();
    MultiGzDecoder::new(fs::File::open(path)?).read_to_end(&mut bytes)?;
    let mut entries = Vec::new();
    let mut at = 0;
    let mut long: Option<String> = None;
    while at + BLOCK <= bytes.len() {
        let block = &bytes[at..at + BLOCK];
        if block.iter().all(|&b| b == 0) {
            break;
        }
        let stored = octal(&block[148..156])?;
        let sum: u64 = block
            .iter()
            .enumerate()
            .map(|(i, &b)| {
                if (148..156).contains(&i) {
                    32
                } else {
                    u64::from(b)
                }
            })
            .sum();
        if stored != sum {
            return Err(invalid("a tar header whose checksum does not match"));
        }
        let size = usize::try_from(octal(&block[124..136])?)
            .map_err(|_| invalid("a tar entry too large"))?;
        let start = at + BLOCK;
        let end = start + size;
        let data = bytes
            .get(start..end)
            .ok_or_else(|| invalid("a tar entry cut short"))?;
        at = end + (BLOCK - size % BLOCK) % BLOCK;
        let name = || {
            let prefix = text(&block[345..500]);
            let name = text(&block[..100]);
            if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            }
        };
        match block[156] {
            b'L' => long = Some(text(data)),
            b'x' => long = pax_path(data).or(long),
            b'0' | 0 => entries.push(Entry {
                path: long.take().unwrap_or_else(name),
                data: data.to_vec(),
            }),
            _ => long = None,
        }
    }
    Ok(entries)
}

/// Writes `entries` under `dir`, refusing any path that leaves it.
pub fn extract(entries: &[Entry], dir: &Path) -> io::Result<()> {
    for entry in entries {
        let relative = PathBuf::from(&entry.path);
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(invalid("an archive path outside its directory"));
        }
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &entry.data)?;
    }
    Ok(())
}

/// Every directory and file under `dir`, sorted, as (archive path, source
/// file; `None` for a directory).
fn walk(dir: &Path, name: &str, out: &mut Vec<(String, Option<PathBuf>)>) -> io::Result<()> {
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = format!("{name}/{}", entry.file_name().to_string_lossy());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            out.push((path.clone(), None));
            walk(&entry.path(), &path, out)?;
        } else if kind.is_file() {
            out.push((path, Some(entry.path())));
        }
    }
    Ok(())
}

/// A ustar header for `path` (`/`-separated, split into prefix and name
/// past 100 bytes) of `size` bytes and `kind` (`b'0'` a file, `b'5'` a
/// directory).
fn header(path: &str, size: u64, kind: u8, mtime: u64) -> io::Result<[u8; BLOCK]> {
    let (prefix, name) = split(path)?;
    let mut block = [0_u8; BLOCK];
    block[..name.len()].copy_from_slice(name.as_bytes());
    let mode = if kind == b'5' { 0o755 } else { 0o644 };
    put_octal(&mut block[100..108], mode);
    put_octal(&mut block[108..116], 0);
    put_octal(&mut block[116..124], 0);
    put_octal(&mut block[124..136], size);
    put_octal(&mut block[136..148], mtime);
    block[156] = kind;
    block[257..263].copy_from_slice(b"ustar\0");
    block[263..265].copy_from_slice(b"00");
    block[345..345 + prefix.len()].copy_from_slice(prefix.as_bytes());
    block[148..156].fill(b' ');
    let sum: u64 = block.iter().map(|&b| u64::from(b)).sum();
    put_octal(&mut block[148..155], sum);
    block[155] = b' ';
    Ok(block)
}

/// `path` as ustar's prefix (at most 155 bytes) and name (at most 100),
/// split at a `/`.
fn split(path: &str) -> io::Result<(&str, &str)> {
    if path.len() <= 100 {
        return Ok(("", path));
    }
    let bytes = path.as_bytes();
    // The name may end in `/` (a directory): split before it.
    let search = path.len().saturating_sub(1);
    (0..search)
        .rev()
        .filter(|&i| bytes[i] == b'/')
        .find(|&i| i <= 155 && path.len() - i - 1 <= 100)
        .map(|i| (&path[..i], &path[i + 1..]))
        .ok_or_else(|| invalid("a path too long for ustar"))
}

/// Writes `value` in octal, zero-padded, NUL-terminated, into `field`.
fn put_octal(field: &mut [u8], value: u64) {
    let digits = format!("{value:0width$o}", width = field.len() - 1);
    field[..digits.len()].copy_from_slice(digits.as_bytes());
    field[digits.len()..].fill(0);
}

/// An octal field's value (spaces and NULs around it ignored).
fn octal(field: &[u8]) -> io::Result<u64> {
    let digits = text(field);
    let digits = digits.trim();
    if digits.is_empty() {
        return Ok(0);
    }
    u64::from_str_radix(digits, 8).map_err(|_| invalid("a tar header with a bad number"))
}

/// A NUL-terminated field as text.
fn text(field: &[u8]) -> String {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// The `path` of a pax extended header's records (`<len> path=<value>\n`).
fn pax_path(data: &[u8]) -> Option<String> {
    String::from_utf8_lossy(data)
        .lines()
        .find_map(|record| record.split_once(" path=").map(|(_, path)| path.to_owned()))
}

/// An error for data that is not what an archive holds.
fn invalid(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, what.to_owned())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use super::{extract, pack, read, split};

    #[test]
    fn a_packed_directory_reads_back_and_tar_lists_it() {
        let root = std::env::temp_dir().join(format!("rustd3-archive-{}", std::process::id()));
        let dir = root.join("20261029T120000Z-quick");
        let deep = format!(
            "{}/raw",
            ["a-rather-long-profile-directory-name"; 3].join("/")
        );
        fs::create_dir_all(dir.join(&deep)).unwrap();
        fs::write(dir.join("results.json"), "{\"format\": 1}\n").unwrap();
        fs::write(dir.join(&deep).join("run.jsonl"), vec![b'x'; 1300]).unwrap();
        fs::write(dir.join("empty.txt"), "").unwrap();
        let archive = root.join("rustd3-bench-quick-20261029T120000Z.tar.gz");
        let size = pack(&dir, &archive).unwrap();
        let partial = root.join("rustd3-bench-quick-20261029T120000Z.tar.gz.partial");
        assert!(size > 0 && !partial.exists());
        let entries = read(&archive).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        let long = format!("20261029T120000Z-quick/{deep}/run.jsonl");
        assert_eq!(
            paths,
            [
                long.as_str(),
                "20261029T120000Z-quick/empty.txt",
                "20261029T120000Z-quick/results.json",
            ]
        );
        assert_eq!(entries[0].data.len(), 1300);
        assert_eq!(entries[2].data, b"{\"format\": 1}\n");
        let into = root.join("extracted");
        extract(&entries, &into).unwrap();
        assert_eq!(fs::read(into.join(&long)).unwrap().len(), 1300);
        // tar itself reads it, where present.
        if let Ok(out) = Command::new("tar").arg("tzf").arg(&archive).output()
            && out.status.success()
        {
            let listed = String::from_utf8_lossy(&out.stdout);
            assert!(listed.lines().any(|line| line == long), "{listed}");
        }
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn long_paths_split_at_a_slash_or_are_refused() {
        assert_eq!(split("a/b").unwrap(), ("", "a/b"));
        let path = format!("{}/{}", "p".repeat(120), "n".repeat(90));
        assert_eq!(split(&path).unwrap(), (&path[..120], &path[121..]));
        assert!(split(&"x".repeat(101)).is_err(), "no slash to split at");
        assert!(split(&format!("{}/{}", "p".repeat(160), "n".repeat(10))).is_err());
    }

    #[test]
    fn paths_that_leave_the_directory_are_refused() {
        let entry = |path: &str| super::Entry {
            path: path.to_owned(),
            data: Vec::new(),
        };
        let dir = std::env::temp_dir().join(format!("rustd3-extract-{}", std::process::id()));
        assert!(extract(&[entry("../escape")], &dir).is_err());
        assert!(extract(&[entry("/etc/passwd")], &dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
