//! Tiny file log: warnings/errors + startup time only. Never log user content.
use std::{fs, io::Write, path::{Path, PathBuf}, sync::OnceLock, time::{SystemTime, UNIX_EPOCH}};

const MAX_BYTES: u64 = 1024 * 1024;
static PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init() {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("DynamicIsland");
    let _ = fs::create_dir_all(&dir);
    let _ = PATH.set(dir.join("app.log"));
}

pub fn info(msg: &str) {
    write("INFO", msg)
}
pub fn warn(msg: &str) {
    write("WARN", msg)
}
pub fn error(msg: &str) {
    write("ERROR", msg)
}

fn write(level: &str, msg: &str) {
    #[cfg(debug_assertions)]
    eprintln!("[{level}] {msg}");
    if let Some(p) = PATH.get() {
        append(p, MAX_BYTES, level, msg);
    }
}

/// Appends a line, rotating to `<file>.1` (single generation) once `max` is reached.
fn append(path: &Path, max: u64, level: &str, msg: &str) {
    if fs::metadata(path).map(|m| m.len() >= max).unwrap_or(false) {
        let _ = fs::rename(path, path.with_extension("log.1"));
    }
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
        // One write per line: `writeln!` issues several, which interleave when the media thread logs too.
        let _ = f.write_all(format!("{ts} {level} {msg}\n").as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_and_caps_disk_usage() {
        let d = std::env::temp_dir().join(format!("di-log-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        let p = d.join("app.log");
        for _ in 0..200 {
            append(&p, 1000, "WARN", "0123456789012345678901234567890123456789");
        }
        let main = fs::metadata(&p).unwrap().len();
        let old = fs::metadata(p.with_extension("log.1")).unwrap().len();
        assert!(main <= 1000 + 64 && old <= 1000 + 64, "main={main} old={old}");
    }
}
