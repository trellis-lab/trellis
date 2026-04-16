use std::fs;
use std::io;
use std::path::Path;

use super::DebugLog;

/// Write a `DebugLog` to `path` as pretty-printed JSON.
///
/// Failure is non-fatal — callers should log a warning and continue rendering.
pub fn write_debug_log(path: &Path, log: &DebugLog) -> io::Result<()> {
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, log).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}
