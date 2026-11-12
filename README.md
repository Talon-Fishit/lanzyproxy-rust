# LanzyProxy (Rust)

A Growtopia proxy: it answers `server_data.php` so the game client connects
here, relays the ENet traffic to the real server, and exposes a set of in-game
commands over that relay.

This is the Rust rewrite of the Python framework in
[`Talon-Fishit/lanzyproxy`](https://github.com/Talon-Fishit/lanzyproxy), with
the command set cross-checked against the reference build's own vocabulary.

Same behaviour, a fraction of the memory:

| | RSS |
| --- | --- |
| Python | 13.5 MB |
| Rust | **1.8 MB** |

## Build

```bash
cargo build --release                       # native
cargo build --release --target x86_64-pc-windows-gnu   # lanzyproxy.exe
```

Cross-compiling to Windows needs `mingw-w64`, `cmake` and `libclang`:

```bash
sudo apt-get install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 cmake libclang-dev
rustup target add x86_64-pc-windows-gnu
LIBCLANG_PATH=/usr/lib/llvm-19/lib cargo build --release --target x86_64-pc-windows-gnu
```

The result is `target/x86_64-pc-windows-gnu/release/lanzyproxy.exe`, about
550 KB with no runtime dependencies beyond the C runtime.

`./build.sh` does the whole thing.

## Run

```
lanzyproxy              run the proxy: spoofer + ENet relay
lanzyproxy --offline    the offline shell: run commands without a game
lanzyproxy --list       print the command list
lanzyproxy --version    print the version
```

Settings live in `config.json` next to the binary, or wherever
`LANZYPROXY_CONFIG` points.

The spoofer needs port 80, which on Windows means running as administrator.

## Layout

| Crate | |
| --- | --- |
| `lzprotocol` | `NetMessage`, `GameUpdatePacket`, `VariantList`, the packet constructors |
| `lzconfig` | the settings document, with unknown keys preserved across a round trip |
| `lzstate` | the session state: world, tiles, players, tracked drops |
| `lzdialog` | the in-game dialog markup builder |
| `lzfeatures` | the command set and the feature modules |
| `lzproxy` | the HTTP spoofer and the ENet relay |
| `lanzyproxy` | the binary |

## Commands

67 commands, 596 aliases. `/proxy` lists them in the game; `--list` prints them
on the terminal.

| Group | |
| --- | --- |
| Utility | `/proxy` `/settings` `/options` `/keep` `/reset` `/hotkey` `/logs` `/server` `/logoff` `/calc` `/switch` `/status` |
| Automation | `/autofarm` `/farm` `/autoplant` `/plant` `/autoharvest` `/harvest` `/autocollect` `/collect` `/autocook` `/startcook` `/fish` `/providers` `/crime` `/acc` `/autolock` `/auto` `/anti` |
| Economy | `/vend` `/fastvend` `/shop` `/dice` `/check` `/gazette` |
| Visuals | `/clothes` `/unequip` `/titles` `/weather` `/country` `/spirit` `/ui` `/speed` |
| Navigation | `/pathfind` `/pfind` `/pvend` `/find` `/render` `/parkour` `/blocks` `/gs` `/items` `/tree` `/chest` |
| World | `/world` `/worlds` `/list` `/scan` |
| Moderation | `/wrench` `/locked` `/lost` `/moddetect` `/track` `/pkickall` `/hidden` |
| Casino | `/roulette` `/showspin` |

## The reference build

The original tool this was rebuilt from is a Windows binary wrapped in a
third-party crypter. Its command vocabulary was recovered statically — the
import table, the export table, and the strings carrying its command names —
and every command it names is covered here, as a command or as an alias.

Recovered and carried over:

- the `NetMessage` / `GameUpdatePacket` / `VariantList` wire format,
- the `server_data.php` response body, including the `RTENDMARKERBS1001` marker,
- the dialog markup directives and their exact spelling,
- the roulette broadcast strings (`spun the wheel and got`, the `REAL` and
  `FAKE` tags, the `REME` / `QEME` / `LEME` suffixes),
- the `AddCommandInfo` command registry.

No code, artwork or data was copied: the original is a licensed product and
this is an independent implementation.

### Checking the match

`scripts/compare_with_reference.py` extracts the registry from the reference
build and checks this port against it. Nothing is executed — the reference is
only read as data.

```
$ python3 scripts/compare_with_reference.py path/to/reference.exe
==============================================================
                  RUST PORT vs REFERENCE BUILD
==============================================================

  command names        62 / 62    OK
  aliases             296 / 296   OK
  dialog directives    30 / 30    OK
  server_data fields    9 / 9      OK
  options menu         17 / 17     OK
```

Five commands are extra: `anti`, `options`, `roulette`, `showspin` and
`status`. The reference exposes the roulette pages through the options menu
instead of as top-level commands, and has no `status` page; these are
additions, not differences.

## Tests

```bash
cargo test
```

Tests cover the wire format, the dialog markup, the settings round trip, the
command registry, the roulette parser and the options menu.

## Licence

MIT.
