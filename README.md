# Libium
Libium is the backend of Ferium. It helps manage Minecraft mods from Modrinth, CurseForge, and Github Releases

There are 3 main components in Libium;

- `config` deals with (surprise, surprise) the config. It defines the config struct and some methods to get the config file, deserialise it, etc
- `misc` contains a few convenience functions
- `file_picker` contains functions to show a file picker
- `add` contains functions to verify and add a mod to a profile
- `modpack` contains manifest/metadata structs and functions for Modrinth and CurseForge structs
- `upgrade` contains functions for the advanced upgrade functionality in Ferium
