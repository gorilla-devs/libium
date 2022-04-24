# Changelog for Libium

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
