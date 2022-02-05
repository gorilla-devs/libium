# Libium
Libium is the backend of Ferium. It helps manage Minecraft mods from Modrinth, CurseForge, and Github Releases

There are 3 main components in Libium;

- `config` is (surprise, surprise) the config for Ferium. It defines the config struct and some methods to get the config file, deserialise it, etc
- `launchermeta` is an api-binding to Mojang's version manifest REST API
- `misc` contains a few convenience functions
