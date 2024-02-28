use crate::{
    config::structs::{Mod, ModIdentifier, Profile},
    upgrade::mod_downloadable,
};

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
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.name.to_lowercase()
            || ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    }) {
        Err(super::Error::AlreadyAdded)

    // Check if it can be downloaded by third-parties
    } else if Some(false) == project.allow_mod_distribution {
        Err(super::Error::DistributionDenied)

    // Check if the project is a Minecraft mod
    } else if !project.links.website_url.as_str().contains("mc-mods") {
        Err(super::Error::NotAMod)

    // Check if the project is compatible
    } else {
        if perform_checks {
            mod_downloadable::get_latest_compatible_file(
                curseforge.get_mod_files(project.id).await?,
                profile.get_version(check_game_version),
                profile.get_loader(check_game_version),
            )
            .ok_or(super::Error::Incompatible)?;
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
}
