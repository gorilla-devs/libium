use ferinth::structures::project::{DonationLink, Project, ProjectType};

use crate::{
    config::structs::{Mod, ModIdentifier, ModIdentifierRef, ModLoader, Profile},
    upgrade::check::{game_version_check, mod_loader_check},
};

use super::Checks;

fn project_exist(profile: &Profile, project: &Project) -> bool {
    profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.title.to_lowercase()
            || ModIdentifierRef::ModrinthProject(&project.id) == mod_.identifier.as_ref()
    })
}

fn project_is_mod(project: &Project) -> bool {
    project.project_type == ProjectType::Mod
}

fn check_mod_loader_fabric_backwards_compatible(
    profile: &Profile,
    project: &Project,
    check_mod_loader: bool,
) -> bool {
    mod_loader_check(profile.get_loader(check_mod_loader), &project.loaders)
        || (profile.mod_loader == ModLoader::Quilt
            && mod_loader_check(Some(ModLoader::Fabric), &project.loaders))
}

fn project_comatible(profile: &Profile, project: &Project, checks: &Checks) -> bool {
    game_version_check(
        profile.get_version(checks.game_version()),
        &project.game_versions,
    ) && check_mod_loader_fabric_backwards_compatible(profile, project, checks.mod_loader())
}

/// Check if the project of `project_id` exists, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
///
/// Returns the project name and donation URLs to display to the user
pub async fn modrinth(
    modrinth: &ferinth::Ferinth,
    project_id: &str,
    profile: &mut Profile,
    checks: &Checks,
) -> super::Result<(String, Vec<DonationLink>)> {
    let project = modrinth.get_project(project_id).await?;

    if project_exist(profile, &project) {
        return Err(super::Error::AlreadyAdded);
    }

    if !project_is_mod(&project) {
        return Err(super::Error::NotAMod);
    }

    if checks.perform_checks() && !project_comatible(profile, &project, checks) {
        return Err(super::Error::Incompatible);
    }

    profile.mods.push(Mod::new(
        project.title.trim(),
        ModIdentifier::ModrinthProject(project.id),
        checks,
    ));

    Ok((project.title, project.donation_urls))
}
