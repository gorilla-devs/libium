use super::{DistributionDeniedError, DownloadFile};
use crate::{
    config::{
        filters::Filter,
        structs::{Mod, ModIdentifier},
    },
    iter_ext::IterExt as _,
    CURSEFORGE_API, GITHUB_API, MODRINTH_API,
};
use std::cmp::Reverse;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    DistributionDenied(#[from] DistributionDeniedError),
    CheckError(#[from] super::check::Error),
    #[error("The pin provided is an invalid identifier")]
    InvalidPinID(#[from] std::num::ParseIntError),
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
    #[error("GitHub: {0:#?}")]
    GitHubError(#[from] octocrab::Error),
}
type Result<T> = std::result::Result<T, Error>;

impl Mod {
    pub async fn fetch_download_file(
        &self,
        mut profile_filters: Vec<Filter>,
    ) -> Result<DownloadFile> {
        if let Some(pin) = &self.pin {
            match &self.identifier {
                ModIdentifier::CurseForgeProject(mod_id) => Ok(CURSEFORGE_API
                    .get_mod_file(*mod_id, pin.parse()?)
                    .await?
                    .try_into()?),
                ModIdentifier::ModrinthProject(_) => {
                    Ok(MODRINTH_API.get_version(pin).await?.into())
                }
                ModIdentifier::GitHubRepository((owner, repo)) => Ok(GITHUB_API
                    .repos(owner, repo)
                    .release_assets()
                    .get(pin.parse()?)
                    .await?
                    .into()),
            }
        } else {
            let download_files = match &self.identifier {
                ModIdentifier::CurseForgeProject(id) => {
                    let mut files = CURSEFORGE_API.get_mod_files(*id).await?;
                    files.sort_unstable_by_key(|f| Reverse(f.file_date));
                    files
                        .into_iter()
                        .map(|x| x.try_into().map_err(Into::into))
                        .collect::<Result<Vec<_>>>()?
                }
                ModIdentifier::ModrinthProject(id) => MODRINTH_API
                    .list_versions(id)
                    .await
                    .map(|x| x.into_iter().map(Into::into).collect_vec())?,
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
                    })?,
            };
            Ok(super::check::select_latest(
                download_files,
                if self.override_filters {
                    self.filters.clone()
                } else {
                    profile_filters.extend(self.filters.clone());
                    profile_filters
                },
            )
            .await?)
        }
    }
}
