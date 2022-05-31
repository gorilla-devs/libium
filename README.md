# Libium
Libium is the backend of Ferium and Carbon. It is responsible for everything platform independent: Minecraft profiles, Java installations and Mod management

### Structure
#### There are 6 main components in Libium:
- `config` deals with (surprise, surprise) the config. It defines the config struct and some methods to get the config file, deserialize it, etc
- `modpacks` has functions for managing entire Modpacks.
- `mods` contains functions to manage mods belonging to a profile.
- `game` downloads and manages different Modloaders and Minecraft game files.
- `java` deals with the different Java applications.
- `launcher` is responsible for launching the game.


- `misc` contains a few convenience functions
- `file_picker` contains functions to show a file picker

<details>
  <summary>
  <b>Full Gdevs Library Structure</b> (click to expand)
  </summary>
  Up to date versions:
  <a href="https://github.com/gorilla-devs/libium/blob/libium-rewrite/static/library-layout.txt">Original</a> -
  <a href="https://github.com/gorilla-devs/libium/blob/libium-rewrite/static/library-layout-compact.txt">Compact version</a>

```
  ╔════════╗ ╔════════╗     | Interfaces for Libium, both as CLI as GUI.
  ║ Carbon ║ ║ Ferium ║     | Ferium: CLI version written using Clap.
  ╚════╤═══╝ ╚═══╤════╝     | Carbon: GUI version using electron, written in SolidJS
       │         │
       ╰────┬────╯
            │
       ╔════╧════╗          | Libium, the library that does all the platform independent
       ║ Libium  ║          | work, both for Mod loaders as for Mod hosting Platforms.
       ╚════╤════╝          | Manages profiles, launches the game, modifies the config...
            │
╭───────────╯
│
│ ┏━━━━━━━━━━━━━━━━━━━━━━━┓ | Extendable Mod loaders, managing Minecraft's inner game
├─┨     Mod loaders       ┃ | files, like metadata, versions and launch commands.
│ ┃╔═══════╗   ╔═════════╗┃ | 
│ ┃║ Faber ║ ∙ ║ Forgic  ║┃ | Vanel:   Vanilla implementation, no mods.
│ ┃╚═══════╝∙∙∙╚═════════╝┃ | Quantum: Manager for the Quilt Mod Loader
│ ┃╔═══════╗∙∙∙╔═════════╗┃ | Faber:   Manager for the Fabric Mod Loader
│ ┃║ Vanel ║ ∙ ║ Quantum ║┃ | Forgic:  Manager for the Forge Mod Loader
│ ┃╚═══════╝   ╚═════════╝┃ | 
│ ┗━━━━━━━━━━┯━━━━━━━━━━━━┛
│        ╔═══╧═══╗          | Ludic, the library providing uniform Mod loader objects,
│        ║ Ludic ║          | that get traits in the Mod loader implementations.
│        ╚═══════╝          |
│
│ ┏━━━━━━━━━━━━━━━━━━━━━━━┓ | Extendable Mod hosting platforms, providing everything from
╰─┨      Platforms        ┃ | Mods, Resource Packs, Modpacks and Worlds to Datapacks,
  ┃╔═══════╗   ╔═════════╗┃ | Server Plugins and Shaders.
  ┃║ Furse ║∙∙∙║ Ferinth ║┃ | 
  ┃╚═══════╝   ╚═════════╝┃ | Furse:   Worker for the CurseForge API
  ┗━━━━━━━━━━┯━━━━━━━━━━━━┛ | Ferinth: Implementation for Modrinth
             │
        ╔════╧════╗         | Dotium, providing uniform Platform objects that then get traits
        ║ Dotium  ║         | in the Platform implementations.
        ╚═════════╝         |
```
</details>

### Development
- Compiling <br/>
  `cargo build`
- Testing <br/>
  `cargo test`
- Developing <br/>
  Compile either [Ferium](https://github.com/gorilla-devs/ferium) (recommended) or [Carbon](https://github.com/gorilla-devs/gdlauncher) with your own version of libium.
