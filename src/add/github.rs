use crate::{
    config::structs::{Mod, ModIdentifier, ModIdentifierRef, Profile},
    upgrade::mod_downloadable,
};

/// Check if the repo of `repo_handler` exists, releases mods, and is compatible with `profile`.
/// If so, add it to the `profile`.
///
/// Returns the name of the repository to display to the user
pub async fn github(
    repo_handler: &octocrab::repos::RepoHandler<'_>,
    profile: &mut Profile,
    perform_checks: bool,
    check_game_version: bool,
    check_mod_loader: bool,
) -> super::Result<String> {
    let repo = repo_handler.get().await?;
    let repo_name = (repo.owner.clone().unwrap().login, repo.name.clone());

    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == repo.name.to_lowercase()
            || ModIdentifierRef::GitHubRepository(&repo_name) == mod_.identifier.as_ref()
    }) {
        return Err(super::Error::AlreadyAdded);
    }

    if perform_checks {
        let releases = repo_handler.releases().list().send().await?.items;

        // Check if jar files are released
        if !releases
            .iter()
            .flat_map(|r| &r.assets)
            .any(|a| a.name.ends_with(".jar"))
        {
            return Err(super::Error::NotAMod);
        }

        // Check if the repo is compatible
        mod_downloadable::get_latest_compatible_asset(
            &releases,
            profile.get_version(check_game_version),
            profile.get_loader(check_game_version),
        )
        .ok_or(super::Error::Incompatible)?;
    }

    // Add it to the profile
    profile.mods.push(Mod {
        name: repo.name.trim().to_string(),
        identifier: ModIdentifier::GitHubRepository(repo_name),
        check_game_version,
        check_mod_loader,
    });

    Ok(repo.name)
}
