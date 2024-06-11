use ferinth::structures::version::{Version, VersionFile};

pub trait VersionExt {
    fn get_version_file(&self) -> &VersionFile;
    fn into_version_file(self) -> VersionFile;
}

impl VersionExt for Version {
    fn get_version_file(&self) -> &VersionFile {
        self.files
            .iter()
            .find(|f| f.primary)
            .unwrap_or(&self.files[0])
    }

    fn into_version_file(self) -> VersionFile {
        let fallback = self.files[0].clone();
        self.files
            .into_iter()
            .find(|f| f.primary)
            .unwrap_or(fallback)
    }
}
