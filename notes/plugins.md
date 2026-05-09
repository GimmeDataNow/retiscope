# Plugins
Retiscope will in all likelyhood have to employ the use of plugins for data parsing purposes.

As it currently stands the plan is as follows:
1. Retiscope will load all plugins
2. It will then add them to a prefix tree
3. It will then take a packet and score each plugin
4. The plugin with the highest score will be run agains the packet
5. The plugin will return the parsed packet as a string

Additionally I had the idea of making the plugin return a spcial struct which then could be used to
highlight additional data in the hex view.

## Plugin design

I am aiming for high performance on the packet parsing due to cumulative performance penalties.
Plugins need to by dynamically loaded.

I think I will have to use wasm for the plugins.

### Reasons
- Cross-platform
- Compiled once
- High stability (hopefully)
- Ease of use (kinda)
- Language Agnostic (this might be big)
- Can be dynamically loaded
