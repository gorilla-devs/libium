# Changelog for Libium

## `1.11.4`
### 08.05.2022

- Do not check the release name when checking the game version for github releases
  - This fixes Ferium [#47](https://github.com/theRookieCoder/ferium/issues/47)

## `1.11.3`
### 05.05.2022

- Added `prompt` to file pickers
- Used the `default` provided to the no-gui pick folder

## `1.11.2`
### 05.05.2022

Change macOS default mods directory from using the `ApplicationSupport` shortcut to the actual `Application Support` directory

## `1.11.1`
### 04.05.2022

- Updated to Ferinth `2.2`
- Add commands now accept `should_check_game_version` and `should_check_mod_loader`
- They also use this when adding the mod to the config

## `1.11.0`
### 03.05.2022

- Replace the `for` loop in `check_mod_loader()` with an iterator call
- The upgrade functions no longer deal with Quilt -> Fabric backwards compatibility
- Upgrade functions (again) return only the compatibile asset they found
- Upgrade functions no longer take a `profile`, they check for compatibility with the `game_version_to_check` and `mod_loader_to_check` provided

## `1.10.0`
### 01.05.2022

- Added minor versions to `Cargo.toml`
- Update to Furse `1.1`
  - Implemented new error type
- Simplified checking if a project had already been added
- `upgrade::github()` now checks that the asset isn't a sources jar

## [1.9.0] - 24.04.2022

- Added Quilt to `ModLoader`
- Added `check_mod_loader()` to check mod loader compatibility
- The upgrade functions now return additional info, whether the mod was deemed compatible through backwards compatibility (e.g. Fabric mod on Quilt)
- Generally improved code in `upgrade`

## [1.8.0] - 20.04.2022

- Added a `check_mod_loader` and `check_game_version` flag to each mod
- They are `None` by default
- If they are `Some(false)` then the corresponding checks are skipped in `upgrade.rs`
- Removed `no_patch_check`, `remove_semver_patch()`, `SemVerError`, and the `semver` dependency

## [1.7.0] - 15.04.2022

- Remove `config` from function names in config module
- Upgrade functions no longer download and write the mod file
- `write_mod_file()`  is now public

## [1.6.0] - 02.04.2022

Update the `config` struct format

## [1.5.0] - 29.03.2022

- Moved `upgrade.rs` from ferium to libium
  - Added improved custom error handling
  - Improved doc comments
  - Made functions return the file/version/asset downloaded (similar to `add.rs`)
  - Changed some variable names

## [1.4.0] - 28.03.2022

- Moved `add.rs` from ferium to libium
  - Added improved custom error handling
- Extracted file dialogues to `file_picker.rs`
