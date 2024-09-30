use crate::{
    config::{
        filters::ReleaseChannel,
        structs::{Mod, ModIdentifier, ModIdentifierRef, ModLoader, Profile},
    },
    iter_ext::IterExt as _,
    upgrade::{check, DownloadFile},
    CURSEFORGE_API, GITHUB_API, MODRINTH_API,
};
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "The developer of this project has denied third party applications from downloading it"
    )]
    /// The user can manually download the mod and place it in the `user` folder of the output directory to mitigate this.
    /// However, they will have to manually update the mod.
    DistributionDenied,
    #[error("The project has already been added")]
    AlreadyAdded,
    #[error("The project is not compatible because {_0}")]
    Incompatible(#[from] check::Error),
    #[error("The project does not exist")]
    DoesNotExist,
    #[error("The project is not a mod")]
    NotAMod,
    #[error("GitHub: {0}")]
    GitHubError(String),
    #[error("GitHub: {0:#?}")]
    OctocrabError(#[from] octocrab::Error),
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Debug)]
struct GraphQlResponse {
    data: HashMap<String, Option<ResponseData>>,
    #[serde(default)]
    errors: Vec<GraphQLError>,
}

#[derive(Deserialize, Debug)]
struct GraphQLError {
    #[serde(rename = "type")]
    type_: String,
    path: Vec<String>,
    message: String,
}

#[derive(Deserialize, Debug)]
struct ResponseData {
    owner: OwnerData,
    name: String,
    releases: ReleaseConnection,
}
#[derive(Deserialize, Debug)]
struct OwnerData {
    login: String,
}
#[derive(Deserialize, Debug)]
struct ReleaseConnection {
    nodes: Vec<Release>,
}
#[derive(Deserialize, Debug)]
struct Release {
    #[serde(rename = "releaseAssets")]
    assets: ReleaseAssetConnection,
}
#[derive(Deserialize, Debug)]
struct ReleaseAssetConnection {
    nodes: Vec<ReleaseAsset>,
}
#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
}

pub fn parse_id(id: String) -> ModIdentifier {
    if let Ok(id) = id.parse() {
        ModIdentifier::CurseForgeProject(id)
    } else {
        let split = id.split('/').collect_vec();
        if split.len() == 2 {
            ModIdentifier::GitHubRepository((split[0].to_owned(), split[1].to_owned()))
        } else {
            ModIdentifier::ModrinthProject(id)
        }
    }
}

/// Adds mods from `identifiers`, and returns successful mods with their names, and unsuccessful mods with an error
///
/// Classifies the `identifiers` into the appropriate platforms, sends batch requests to get the necessary information,
/// checks details about the projects, and adds them to `profile` if suitable.
/// Performs checks on the mods to see whether they're compatible with the profile if `perform_checks` is true
pub async fn add(
    profile: &mut Profile,
    identifiers: Vec<ModIdentifier>,
    perform_checks: bool,
) -> Result<(Vec<String>, Vec<(String, Error)>)> {
    let mut mr_ids = Vec::new();
    let mut cf_ids = Vec::new();
    let mut gh_ids = Vec::new();
    let mut errors = Vec::new();

    for id in identifiers {
        match id {
            ModIdentifier::CurseForgeProject(id) => cf_ids.push(id),
            ModIdentifier::ModrinthProject(id) => mr_ids.push(id),
            ModIdentifier::GitHubRepository(id) => gh_ids.push(id),
        }
    }

    let cf_projects = if !cf_ids.is_empty() {
        cf_ids.sort_unstable();
        cf_ids.dedup();
        CURSEFORGE_API.get_mods(cf_ids.clone()).await?
    } else {
        Vec::new()
    };

    let mr_projects = if !mr_ids.is_empty() {
        mr_ids.sort_unstable();
        mr_ids.dedup();
        MODRINTH_API
            .get_multiple_projects(&mr_ids.iter().map(AsRef::as_ref).collect_vec())
            .await?
    } else {
        Vec::new()
    };

    let gh_repos = {
        // Construct GraphQl query using raw strings
        let mut graphql_query = "{".to_string();
        for (i, (owner, name)) in gh_ids.iter().enumerate() {
            graphql_query.push_str(&format!(
                "_{i}: repository(owner: \"{owner}\", name: \"{name}\") {{
                    owner {{
                        login
                    }}
                    name
                    releases(first: 100) {{
                      nodes {{
                        releaseAssets(first: 10) {{
                          nodes {{
                            name
                          }}
                        }}
                      }}
                    }}
                }}"
            ));
        }
        graphql_query.push('}');

        // Send the query
        let response: GraphQlResponse = if !gh_ids.is_empty() {
            GITHUB_API
                .graphql(&HashMap::from([("query", graphql_query)]))
                .await?
        } else {
            GraphQlResponse {
                data: HashMap::new(),
                errors: Vec::new(),
            }
        };

        errors.extend(response.errors.into_iter().map(|v| {
            (
                {
                    let id = &gh_ids[v.path[0]
                        .strip_prefix('_')
                        .and_then(|s| s.parse::<usize>().ok())
                        .expect("Unexpected response data")];
                    format!("{}/{}", id.0, id.1)
                },
                if v.type_ == "NOT_FOUND" {
                    Error::DoesNotExist
                } else {
                    Error::GitHubError(v.message)
                },
            )
        }));

        response
            .data
            .into_values()
            .flatten()
            .map(|d| {
                (
                    (d.owner.login, d.name),
                    d.releases
                        .nodes
                        .into_iter()
                        .flat_map(|r| r.assets.nodes.into_iter().map(|e| e.name))
                        .collect_vec(),
                )
            })
            .collect_vec()
    };

    let mut success_names = Vec::new();

    for project in cf_projects {
        if let Some(i) = cf_ids.iter().position(|&id| id == project.id) {
            cf_ids.swap_remove(i);
        }

        match curseforge(&project, profile, perform_checks).await {
            Ok(_) => success_names.push(project.name),
            Err(err) => errors.push((format!("{} ({})", project.name, project.id), err)),
        }
    }
    errors.extend(
        cf_ids
            .iter()
            .map(|id| (id.to_string(), Error::DoesNotExist)),
    );

    for project in mr_projects {
        if let Some(i) = mr_ids
            .iter()
            .position(|id| id == &project.id || project.slug.eq_ignore_ascii_case(id))
        {
            mr_ids.swap_remove(i);
        }

        match modrinth(&project, profile, perform_checks).await {
            Ok(_) => success_names.push(project.title),
            Err(err) => errors.push((format!("{} ({})", project.title, project.id), err)),
        }
    }
    errors.extend(
        mr_ids
            .iter()
            .map(|id| (id.to_string(), Error::DoesNotExist)),
    );

    for (repo, asset_names) in gh_repos {
        match github(&repo, profile, Some(&asset_names)).await {
            Ok(_) => success_names.push(format!("{}/{}", repo.0, repo.1)),
            Err(err) => errors.push((format!("{}/{}", repo.0, repo.1), err)),
        }
    }

    Ok((success_names, errors))
}

/// Check if the repo of `repo_handler` exists, releases mods, and is compatible with `profile`.
/// If so, add it to the `profile`.
///
/// Returns the name of the repository to display to the user
pub async fn github(
    id: &(impl AsRef<str> + ToString, impl AsRef<str> + ToString),
    profile: &mut Profile,
    perform_checks: Option<&[String]>,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.eq_ignore_ascii_case(id.1.as_ref())
            || ModIdentifierRef::GitHubRepository((id.0.as_ref(), id.1.as_ref()))
                == mod_.identifier.as_ref()
    }) {
        return Err(Error::AlreadyAdded);
    }

    if let Some(asset_names) = perform_checks {
        // Check if jar files are released
        if !asset_names.iter().any(|name| name.ends_with(".jar")) {
            return Err(Error::NotAMod);
        }

        // Check if the repo is compatible
        check::select_latest(
            asset_names
                .iter()
                .map(|a| DownloadFile {
                    game_versions: a
                        .strip_suffix(".jar")
                        .unwrap_or("")
                        .split('-')
                        .map(ToOwned::to_owned)
                        .collect_vec(),
                    loaders: a
                        .strip_suffix(".jar")
                        .unwrap_or("")
                        .split('-')
                        .filter_map(|s| ModLoader::from_str(s).ok())
                        .collect_vec(),
                    channel: ReleaseChannel::Alpha,
                    download_url: "https://example.com".parse().unwrap(),
                    output: ".jar".into(),
                    length: 0,
                })
                .collect_vec(),
            profile.filters.clone(),
        )
        .await?;
    }

    // Add it to the profile
    profile.mods.push(Mod {
        name: id.1.as_ref().trim().to_string(),
        identifier: ModIdentifier::GitHubRepository((id.0.to_string(), id.1.to_string())),
        pin: None,
        override_filters: false,
        filters: vec![],
    });

    Ok(())
}

use ferinth::structures::project::{Project, ProjectType};

/// Check if the project of `project_id` has not already been added, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
pub async fn modrinth(
    project: &Project,
    profile: &mut Profile,
    perform_checks: bool,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.eq_ignore_ascii_case(&project.title)
            || ModIdentifierRef::ModrinthProject(&project.id) == mod_.identifier.as_ref()
    }) {
        Err(Error::AlreadyAdded)

    // Check if the project is a mod
    } else if project.project_type != ProjectType::Mod {
        Err(Error::NotAMod)

    // Check if the project is compatible
    } else {
        if perform_checks {
            check::select_latest(
                vec![DownloadFile {
                    game_versions: project.game_versions.clone(),
                    loaders: project
                        .loaders
                        .iter()
                        .filter_map(|s| ModLoader::from_str(s).ok())
                        .collect_vec(),
                    channel: ReleaseChannel::Release,
                    download_url: "https://example.com".parse().unwrap(),
                    output: ".jar".into(),
                    length: 0,
                }],
                profile.filters.clone(),
            )
            .await?;
        }
        // Add it to the profile
        profile.mods.push(Mod {
            name: project.title.trim().to_owned(),
            identifier: ModIdentifier::ModrinthProject(project.id.clone()),
            pin: None,
            override_filters: false,
            filters: vec![],
        });
        Ok(())
    }
}

/// Check if the mod of `project_id` has not already been added, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
pub async fn curseforge(
    project: &furse::structures::mod_structs::Mod,
    profile: &mut Profile,
    perform_checks: bool,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.eq_ignore_ascii_case(&project.name)
            || ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)

    // Check if it can be downloaded by third-parties
    } else if Some(false) == project.allow_mod_distribution {
        Err(Error::DistributionDenied)

    // Check if the project is a Minecraft mod
    } else if !project.links.website_url.as_str().contains("mc-mods") {
        Err(Error::NotAMod)

    // Check if the mod is compatible
    } else {
        if perform_checks {
            check::select_latest(
                vec![DownloadFile {
                    game_versions: project
                        .latest_files_indexes
                        .iter()
                        .map(|i| i.game_version.clone())
                        .collect_vec(),
                    loaders: project
                        .latest_files_indexes
                        .iter()
                        .filter_map(|i| {
                            i.mod_loader
                                .as_ref()
                                .and_then(|l| ModLoader::from_str(&format!("{:?}", l)).ok())
                        })
                        .collect_vec(),
                    channel: ReleaseChannel::Release,
                    download_url: "https://example.com".parse().unwrap(),
                    output: ".jar".into(),
                    length: 0,
                }],
                profile.filters.clone(),
            )
            .await?;
        }
        profile.mods.push(Mod {
            name: project.name.trim().to_string(),
            identifier: ModIdentifier::CurseForgeProject(project.id),
            pin: None,
            override_filters: false,
            filters: vec![],
        });

        Ok(())
    }
}
