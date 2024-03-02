use octocrab::models::{repos::Release, Repository};

use crate::{
    config::structs::{Mod, ModIdentifier, ModIdentifierRef, Profile},
    upgrade::mod_downloadable,
};

use super::Checks;

fn project_exist(profile: &Profile, repo: &Repository, repo_name: &(String, String)) -> bool {
    profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == repo.name.to_lowercase()
            || ModIdentifierRef::GitHubRepository(repo_name) == mod_.identifier.as_ref()
    })
}

fn contains_jar_asset(releases: &[Release]) -> bool {
    releases
        .iter()
        .flat_map(|r| &r.assets)
        .any(|a| a.name.ends_with(".jar"))
}

async fn is_project_compatible(
    profile: &Profile,
    releases: &[Release],
    check_game_version: bool,
) -> super::Result<bool> {
    Ok(mod_downloadable::get_latest_compatible_asset(
        releases,
        profile.get_version(check_game_version),
        profile.get_loader(check_game_version),
    )
    .is_some())
}

/// Check if the repo of `repo_handler` exists, releases mods, and is compatible with `profile`.
/// If so, add it to the `profile`.
///
/// Returns the name of the repository to display to the user
pub async fn github(
    repo_handler: &octocrab::repos::RepoHandler<'_>,
    profile: &mut Profile,
    checks: &Checks,
) -> super::Result<String> {
    let repo = repo_handler.get().await?;
    let repo_name = (
        repo.owner
            .clone()
            .expect("Owner name not found in git repo")
            .login,
        repo.name.clone(),
    );

    // Check if project has already been added
    if project_exist(profile, &repo, &repo_name) {
        return Err(super::Error::AlreadyAdded);
    }

    if checks.contains(Checks::ENABLED) {
        let releases = repo_handler.releases().list().send().await?.items;

        // Check if jar files are released
        if !contains_jar_asset(&releases) {
            return Err(super::Error::NotAMod);
        }

        // Check if the repo is compatible
        if !is_project_compatible(profile, &releases, checks.contains(Checks::GAME_VERSION)).await?
        {
            return Err(super::Error::Incompatible);
        }
    }

    // Add it to the profile
    profile.mods.push(Mod::new(
        repo.name.trim(),
        ModIdentifier::GitHubRepository(repo_name),
        checks,
    ));

    Ok(repo.name)
}
