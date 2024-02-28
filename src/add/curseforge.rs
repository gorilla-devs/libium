use crate::{
    config::structs::{Mod, ModIdentifier, Profile},
    upgrade::mod_downloadable,
};

fn project_exist(profile: &Profile, project: &furse::structures::mod_structs::Mod) -> bool {
    profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.name.to_lowercase()
            || ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    })
}

fn distrubution_denied(project: &furse::structures::mod_structs::Mod) -> bool {
    project.allow_mod_distribution.map_or(false, |b| !b)
}

fn is_minecraft_mod(project: &furse::structures::mod_structs::Mod) -> bool {
    project.links.website_url.as_str().contains("mc-mods")
}

async fn is_project_compatible(
    curseforge: &furse::Furse,
    project: &furse::structures::mod_structs::Mod,
    profile: &Profile,
    check_game_version: bool,
) -> super::Result<bool> {
    Ok(mod_downloadable::get_latest_compatible_file(
        curseforge.get_mod_files(project.id).await?,
        profile.get_version(check_game_version),
        profile.get_loader(check_game_version),
    )
    .is_some())
}

/// Check if the mod of `project_id` exists, is a mod, and is compatible with `profile`.
/// If so, add it to the `profile`.
///
/// Returns the mod name to display to the user
pub async fn curseforge(
    curseforge: &furse::Furse,
    project_id: i32,
    profile: &mut Profile,
    perform_checks: bool,
    check_game_version: bool,
    check_mod_loader: bool,
) -> super::Result<String> {
    let project = curseforge.get_mod(project_id).await?;

    // Check if project has already been added
    if project_exist(profile, &project) {
        return Err(super::Error::AlreadyAdded);
    }

    // Check if it can be downloaded by third-parties
    if distrubution_denied(&project) {
        return Err(super::Error::DistributionDenied);
    }

    // Check if the project is a Minecraft mod
    if !is_minecraft_mod(&project) {
        return Err(super::Error::NotAMod);
    }

    // Check if the project is compatible
    if perform_checks
        && !is_project_compatible(curseforge, &project, profile, check_game_version).await?
    {
        return Err(super::Error::Incompatible);
    }

    // Add it to the profile
    profile.mods.push(Mod {
        name: project.name.trim().to_string(),
        identifier: ModIdentifier::CurseForgeProject(project.id),
        check_game_version,
        check_mod_loader,
    });

    Ok(project.name)
}
