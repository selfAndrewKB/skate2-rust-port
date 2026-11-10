# macOS setup (Apple silicon)

Done on 2026-11-09.

## Toolchain

- Rust via rustup, minimal profile → `~/.cargo/bin`.
- CMake and Ninja via `python3 -m pip install --user cmake ninja` → `~/Library/Python/3.9/bin`
  (installed before Homebrew was available; `brew install cmake ninja` is the normal route).
- Xcode command line tools were already present.

Neither bin folder is on PATH in a fresh shell:

```sh
export PATH="$HOME/.cargo/bin:$HOME/Library/Python/3.9/bin:$PATH"
```

## Game data

`Skate2.iso` is an Xbox 360 XGD2 image. Extracted with extract-xiso (XboxDev), built from source:

```sh
git clone https://github.com/XboxDev/extract-xiso.git
cc -O2 -D__DARWIN__ -D_FILE_OFFSET_BITS=64 -o extract-xiso extract-xiso/extract-xiso.c
./extract-xiso -x Skate2.iso -d Skate2      # note: creates Skate2/Skate2/, flatten it
```

Result: 151 files, 7,091,765,797 bytes, matching the ISO listing. The port can also read the
`.iso` in place, so extraction is optional.

## Running

- `dist/macos/Skate 2.app` after `./build-release-macos.sh game` (about 6 minutes).
- `~/Library/Application Support/Skate 2/config.json` holds `game_root`.
- Launched from Finder there is no console; run `Skate 2.app/Contents/MacOS/Skate 2` from a
  terminal to see output. The welcome/error message boxes don't show on recent macOS; errors go
  to `saves/crash_reports`.
