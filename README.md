# Cluedo

A DDR GP hook that allows you to follow the judgments when playing by a window injected in the game or an OBS overlay.

## Features
- Follow the judgments in real time ingame
- Customizable HTML OBS overlay

## Installation
1. Download the latest release from the [releases page](https://github.com/WenniDev/cluedo/releases)
2. Put it in your game installation root directory (next to `ddr-konaste.exe`)
3. When you start the game, inject the DLL into the process
---
For the OBS overlay:
1. Start gateway.exe and setup the DDR and OBS urls
2. Create a file cluedo.conf next to the cluedo.dll with the following content: (WIP)
```[overlay]
url=http://localhost:7878
```
3. Open `overlay.html` and set the corresponding URL
4. Add a browser source in OBS and select the `overlay.html` file

## License
MIT
