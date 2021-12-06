use std::collections::BTreeSet;
use std::path::Path;
use std::ffi::OsString;


#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct DirEntries {
    pub dirs: BTreeSet<OsString>,
    pub files: BTreeSet<OsString>
}

pub fn dir_entries<P: AsRef<Path>>(dir: P) -> std::io::Result<DirEntries> {
    let mut entries = DirEntries::default();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            entries.dirs.insert(entry.file_name());
        } else {
            entries.files.insert(entry.file_name());
        }
    }
    Ok(entries)
}
