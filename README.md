# lumatone-rs

> Tools for controlling the Lumatone isomorphic keyboard.

This repo contains a work-in-progress library and GUI app for controlling the [Lumatone](https://lumatone.io), a super cool MIDI keyboard with 280 hexagonal keys arranged in a regular pattern.

This work is a continuation of the [lumachromatic](https://github.com/latentspacecraft/lumachromatic) app I wrote a while back. I decided to implement the MIDI bits in rust, partly to get better at rust, but also to make it easier to bundle a standalone desktop app without getting Electron into the mix.

This project is not yet stable, and I'm likely to push breaking changes to the `main` branch frequently until things settle down a bit.

## Status

Mostly working:

- [x] MIDI driver
  - all command and response types are implemented, but some have not yet been tested against the device
- [x] Lumatone preset files (`.ltn`)
  - Can parse `.ltn` files to a `LumatoneKeyMap` struct
  - `LumatoneKeyMap::to_midi_commands()` returns the commands to send to the device
- [-] Command line tool
  - [x] Sends `.ltn` preset files to the device

On the horizon:

- define the data model for tunings and scales
- replace dioxus gui with tauri & port UI code from [lumachromatic](https://github.com/latentspacecraft/lumachromatic) over.
- implement control api
  - used either via tauri's IPC bridge, or headless via websocket (or similar)
