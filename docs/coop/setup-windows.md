# Windows setup

To be filled in from the Windows PC.

- Toolchain: stable Rust with the MSVC toolchain; build with `.\build-release.bat` in
  `skate2-rust-port`, or use the release zip from GitHub.
- Game data: extract the disc image (extract-xiso has Windows builds) to a folder holding
  `default.xex` and `data/`; `config.json` beside the executable records it as `game_root`.
- Note any machine-specific paths or quirks here, the same way `setup-macos.md` does.
