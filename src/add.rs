use crate::{
    config::structs::{Mod, ModIdentifier, ModIdentifierRef, ModLoader, Profile},
    upgrade::check::{self, game_version_check, mod_loader_check},
    APIs,
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
    #[error("The project is not compatible")]
    Incompatible,
    #[error("The project does not exist")]
    DoesNotExist,
    #[error("The project is not a mod")]
    NotAMod,
    #[error("GitHub: {0}")]
    GitHubError(String),
    #[error("GitHub: {0}")]
    OctocrabError(#[from] octocrab::Error),
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize)]
struct GraphQlResponse {
    data: HashMap<String, Option<ResponseData>>,
    errors: Vec<GraphQLError>,
}

#[derive(Deserialize)]
struct GraphQLError {
    #[serde(rename = "type")]
    _type: String,
    path: Vec<String>,
    message: String,
}

#[derive(Deserialize)]
struct ResponseData {
    owner: OwnerData,
    name: String,
    releases: ReleaseConnection,
}
#[derive(Deserialize)]
struct OwnerData {
    login: String,
}
#[derive(Deserialize)]
struct ReleaseConnection {
    nodes: Vec<Release>,
}
#[derive(Deserialize)]
struct Release {
    #[serde(rename = "releaseAssets")]
    assets: ReleaseAssetConnection,
}
#[derive(Deserialize)]
struct ReleaseAssetConnection {
    nodes: Vec<ReleaseAsset>,
}
#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
}

pub fn parse_id(id: &str) -> ModIdentifierRef<'_> {
    if let Ok(id) = id.parse() {
        ModIdentifierRef::CurseForgeProject(id)
    } else {
        let split = id.split('/').collect::<Vec<_>>();
        if split.len() == 2 {
            ModIdentifierRef::GitHubRepository((split[0], split[1]))
        } else {
            ModIdentifierRef::ModrinthProject(id)
        }
    }
}

/// Classify the `identifiers` into the appropriate platforms, send batch requests to get the necessary information,
/// check details about the projects, and add them to `profile` if suitable.
/// Performs checks on the mods to see whether they're compatible with the profile if `perform_checks` is true
pub async fn add(
    apis: APIs<'_>,
    profile: &mut Profile,
    identifiers: Vec<String>,
    perform_checks: bool,
) -> Result<(Vec<String>, Vec<(String, Error)>)> {
    let mut mr_ids = Vec::new();
    let mut cf_ids = Vec::new();
    let mut gh_ids = Vec::new();
    let mut errors = Vec::new();

    for id in &identifiers {
        match parse_id(id) {
            ModIdentifierRef::CurseForgeProject(id) => cf_ids.push(id),
            ModIdentifierRef::ModrinthProject(id) => mr_ids.push(id),
            ModIdentifierRef::GitHubRepository(id) => gh_ids.push(id),
        }
    }

    let cf_projects = if !cf_ids.is_empty() {
        apis.cf.get_mods(cf_ids.clone()).await?
    } else {
        Vec::new()
    };

    let mr_projects = if !mr_ids.is_empty() {
        apis.mr.get_multiple_projects(&mr_ids).await?
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
            apis.gh
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
                    let id = gh_ids[v.path[0]
                        .strip_prefix('_')
                        .and_then(|s| s.parse::<usize>().ok())
                        .expect("Unexpected response data")];
                    format!("{}/{}", id.0, id.1)
                },
                if v._type == "NOT_FOUND" {
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
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };

    let mut success_names = Vec::new();

    for project in cf_projects {
        if let Some(i) = cf_ids.iter().position(|&id| id == project.id) {
            cf_ids.swap_remove(i);
        }

        match curseforge(&project, profile, perform_checks, true, true) {
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
            .position(|&id| id == project.id || project.slug.eq_ignore_ascii_case(id))
        {
            mr_ids.swap_remove(i);
        }

        match modrinth(&project, profile, perform_checks, true, true) {
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
        match github(&repo, profile, Some(&asset_names), true, true) {
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
pub fn github(
    id: &(impl AsRef<str> + ToString, impl AsRef<str> + ToString),
    profile: &mut Profile,
    perform_checks: Option<&[String]>,
    check_game_version: bool,
    check_mod_loader: bool,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == id.1.as_ref().to_lowercase()
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
        check::github(
            asset_names,
            profile.get_version(check_game_version),
            profile.get_loader(check_game_version),
        )
        .ok_or(Error::Incompatible)?;
    }

    // Add it to the profile
    profile.mods.push(Mod {
        name: id.1.as_ref().trim().to_string(),
        identifier: ModIdentifier::GitHubRepository((id.0.to_string(), id.1.to_string())),
        check_game_version,
        check_mod_loader,
    });

    Ok(())
}

use ferinth::structures::project::{Project, ProjectType};

/// Check if the project of `project_id` has not already been added, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
pub fn modrinth(
    project: &Project,
    profile: &mut Profile,
    perform_checks: bool,
    check_game_version: bool,
    check_mod_loader: bool,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.title.to_lowercase()
            || ModIdentifierRef::ModrinthProject(&project.id) == mod_.identifier.as_ref()
    }) {
        Err(Error::AlreadyAdded)

    // Check if the project is a mod
    } else if project.project_type != ProjectType::Mod {
        Err(Error::NotAMod)

    // Check if the project is compatible
    } else if !perform_checks // Short circuit if the checks should not be performed
        || (
            game_version_check(
                profile.get_version(check_game_version).as_ref(),
                &project.game_versions,
            ) && (
                mod_loader_check(
                    profile.get_loader(check_mod_loader),
                    &project.loaders
                ) || (
                // Fabric backwards compatibility in Quilt
                profile.mod_loader == ModLoader::Quilt
                    && mod_loader_check(Some(ModLoader::Fabric), &project.loaders)
                )
            )
        )
    {
        // Add it to the profile
        profile.mods.push(Mod {
            name: project.title.trim().to_string(),
            identifier: ModIdentifier::ModrinthProject(project.id.clone()),
            check_game_version,
            check_mod_loader,
        });

        Ok(())
    } else {
        Err(Error::Incompatible)
    }
}

/// Check if the mod of `project_id` has not already been added, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
pub fn curseforge(
    project: &furse::structures::mod_structs::Mod,
    profile: &mut Profile,
    perform_checks: bool,
    check_game_version: bool,
    check_mod_loader: bool,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.name.to_lowercase()
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
    } else if !perform_checks // Short-circuit if checks do not have to be performed

        // Extract game version and loader pairs from the 'latest files',
        // which generally exist for every supported game version and loader combination
        || {
            let version = profile.get_version(check_game_version);
            let loader = profile.get_loader(check_mod_loader);
            project
                .latest_files_indexes
                .iter()
                .map(|f| {
                    (
                        &f.game_version,
                        f.mod_loader
                            .as_ref()
                            .and_then(|l| ModLoader::from_str(&format!("{:?}", l)).ok()),
                    )
                })
                .any(|p| {
                    (version.is_none() || version == Some(p.0)) &&
                    (loader.is_none() || loader == p.1)
                })
        }
    {
        profile.mods.push(Mod {
            name: project.name.trim().to_string(),
            identifier: ModIdentifier::CurseForgeProject(project.id),
            check_game_version,
            check_mod_loader,
        });

        Ok(())
    } else {
        Err(Error::Incompatible)
    }
}
