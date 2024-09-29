use super::DownloadFile;
use crate::{
    config::filters::{Filter, ReleaseChannel},
    iter_ext::IterExt,
    MODRINTH_API,
};
use ferinth::structures::tag::GameVersionType;
use regex::Regex;
use std::{collections::HashSet, sync::OnceLock};

static VERSION_GROUPS: OnceLock<Vec<Vec<String>>> = OnceLock::new();

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum Error {
    VersionGrouping(#[from] ferinth::Error),
    FilenameRegex(#[from] regex::Error),
    #[error("The following filter(s) were empty")]
    FilterEmpty(Vec<String>),
    #[error("Failed to find a compatible combination")]
    IntersectFailure,
}
pub type Result<T> = std::result::Result<T, Error>;

pub async fn get_version_groups() -> Result<&'static Vec<Vec<String>>> {
    if let Some(v) = VERSION_GROUPS.get() {
        Ok(v)
    } else {
        let versions = MODRINTH_API.list_game_versions().await?;
        let mut v = vec![vec![]];
        for version in versions {
            if version.version_type == GameVersionType::Release {
                v.last_mut().unwrap().push(version.version);
                if version.major {
                    v.push(vec![]);
                }
            }
        }

        let _ = VERSION_GROUPS.set(v);
        Ok(dbg!(VERSION_GROUPS.get().unwrap()))
    }
}

impl Filter {
    pub async fn filter(&self, download_files: &[DownloadFile]) -> Result<HashSet<usize>> {
        Ok(match self {
            Filter::ModLoaderPrefer(loaders) => loaders
                .iter()
                .map(|l| {
                    download_files
                        .iter()
                        .positions(|f| f.loaders.contains(l))
                        .collect_hashset()
                })
                .find(|v| !v.is_empty())
                .unwrap_or_default(),

            Filter::ModLoaderAny(loaders) => download_files
                .iter()
                .positions(|f| loaders.iter().any(|l| f.loaders.contains(l)))
                .collect_hashset(),

            Filter::GameVersionStrict(versions) => download_files
                .iter()
                .positions(|f| {
                    versions.iter().any(|vc| {
                        f.game_versions
                            .iter()
                            .any(|vi| vi.trim_start_matches("mc") == vc)
                    })
                })
                .collect_hashset(),

            Filter::GameVersionMinor(versions) => {
                let mut final_versions = vec![];
                for group in get_version_groups().await? {
                    if group.iter().any(|v| versions.contains(v)) {
                        final_versions.extend(group.clone());
                    }
                }

                download_files
                    .iter()
                    .positions(|f| {
                        final_versions.iter().any(|vc| {
                            f.game_versions
                                .iter()
                                .any(|vi| vi.trim_start_matches("mc") == vc)
                        })
                    })
                    .collect_hashset()
            }

            Filter::ReleaseChannel(channel) => download_files
                .iter()
                .positions(|f| match channel {
                    ReleaseChannel::Alpha => true,
                    ReleaseChannel::Beta => {
                        f.channel == ReleaseChannel::Beta || f.channel == ReleaseChannel::Release
                    }
                    ReleaseChannel::Release => f.channel == ReleaseChannel::Release,
                })
                .collect_hashset(),
            Filter::Filename(regex) => {
                let regex = Regex::new(regex)?;
                download_files
                    .iter()
                    .positions(|f| regex.is_match(&f.filename()))
                    .collect_hashset()
            }
        })
    }
}

/// Assumes that the provided `download_files` are sorted in chronological order
pub async fn select_latest(
    download_files: Vec<DownloadFile>,
    filters: &[Filter],
) -> Result<DownloadFile> {
    let mut filter_results = vec![];

    for filter in filters {
        filter_results.push((filter, filter.filter(&download_files).await?));
    }

    let empty_filtrations = filter_results
        .iter()
        .filter_map(|(name, indices)| if indices.is_empty() { Some(name) } else { None })
        .collect_vec();
    if !empty_filtrations.is_empty() {
        return Err(Error::FilterEmpty(
            empty_filtrations
                .iter()
                .map(ToString::to_string)
                .collect_vec(),
        ));
    }

    let final_index = filter_results
        .into_iter()
        .map(|(_, set)| set)
        .fold(HashSet::new(), |set_a, set_b| {
            set_a.intersection(&set_b).copied().collect_hashset()
        })
        .into_iter()
        .min()
        .ok_or(Error::IntersectFailure)?;

    Ok(download_files[final_index].clone())
}
