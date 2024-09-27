use super::{DistributionDeniedError, DownloadFile};
use crate::{
    config::structs::ModIdentifier, iter_ext::IterExt as _, CURSEFORGE_API, GITHUB_API,
    MODRINTH_API,
};
use std::cmp::Reverse;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DistributionDenied(#[from] DistributionDeniedError),
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
    #[error("GitHub: {0:#?}")]
    GitHubError(#[from] octocrab::Error),
}
type Result<T> = std::result::Result<T, Error>;

impl ModIdentifier {
    pub async fn fetch_download_files(&self) -> Result<Vec<DownloadFile>> {
        match self {
            ModIdentifier::CurseForgeProject(id) => {
                let mut files = CURSEFORGE_API.get_mod_files(*id).await?;
                files.sort_unstable_by_key(|f| Reverse(f.file_date));
                files
                    .into_iter()
                    .map(|x| x.try_into().map_err(Into::into))
                    .collect::<Result<Vec<_>>>()
            }
            ModIdentifier::ModrinthProject(id) => MODRINTH_API
                .list_versions(id)
                .await
                .map(|x| x.into_iter().map(Into::into).collect_vec())
                .map_err(Into::into),
            ModIdentifier::GitHubRepository((owner, repo)) => GITHUB_API
                .repos(owner, repo)
                .releases()
                .list()
                .send()
                .await
                .map(|r| {
                    r.items
                        .into_iter()
                        .flat_map(|r| r.assets)
                        .map(Into::into)
                        .collect_vec()
                })
                .map_err(Into::into),
        }
    }
}
