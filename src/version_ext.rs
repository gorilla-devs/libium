use ferinth::structures::version_structs::{Version, VersionFile};

pub trait VersionExt {
    fn get_version_file(&self) -> &VersionFile;
    fn into_version_file(self) -> VersionFile;
}

impl VersionExt for Version {
    fn get_version_file(&self) -> &VersionFile {
        for file in &self.files {
            if file.primary {
                return file;
            }
        }
        &self.files[0]
    }

    fn into_version_file(self) -> VersionFile {
        let mut files = Vec::new();
        for file in self.files {
            if file.primary {
                return file;
            } else {
                files.push(file)
            }
        }
        files.swap_remove(0)
    }
}
