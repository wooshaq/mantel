# Mantel

A console-style home screen for Windows handhelds. Mantel puts every game you have
installed (Steam, Epic, GOG, Xbox / Game Pass, Ubisoft Connect and anything else) on one
fullscreen shelf you drive with the controller. Games still launch through their own
stores, so nothing gets reinstalled or reconfigured.

Built on and for the ROG Xbox Ally X; it should feel at home on any Windows handheld or a
TV PC.

![Home](docs/screens/home.jpg)

| Library | Stores | Game details |
| --- | --- | --- |
| ![Library](docs/screens/library.jpg) | ![Stores](docs/screens/store.jpg) | ![Details](docs/screens/details.jpg) |

## Features

- **Works on first launch.** Mantel finds installed games on its own: no plugins, no
  per-store setup.
- **Controller first.** Every screen works with the gamepad, the touchscreen or a keyboard.
  The control center (≡) sets volume and brightness, shows Wi-Fi and battery, closes
  games, restarts or powers off the device. Power off and restart are forced by default
  (Windows' `shutdown.exe /s /f /t 0`); their switches can be changed in Settings → System,
  limited to shutdown's own switches for this PC. Several games and programs can run at once;
  each one closes from its own card.
- **Playtime that adds up.** Mantel times every session itself, games started outside it
  included, and takes Steam's own playtime from its files.
- **No sign-ins.** This build (wooshaq) signs in to no account anywhere: no Steam, Xbox or
  Microsoft sign-in windows, and no tokens on disk.
- **One library, two stores.** The Library holds the games installed from every store, with a
  filter for each store. The Xbox tab is the PC Game Pass catalog and the Microsoft Store,
  the Steam tab the Steam store, each with lists by genre and shelves that open into full
  grids. Search reaches both stores too. Buying happens in the store itself.
- **Install from the couch.** Steam games install through Steam's dialog, with progress on the
  home screen. In this build (wooshaq) Game Pass and Store games open on their page in the Xbox
  app, which installs them, and uninstall in Windows Settings: Mantel never runs PowerShell.
- **In your language.** English, French, German, Polish, Spanish and Russian.
- **Achievements.** Played Steam games show their progress from your public Steam profile,
  and a game's page lists each achievement with its rarity and the date you unlocked it
  (Settings → Library → Steam achievements). No sign-in: the profile's game details only
  have to be public.
- **Statistics.** The first tab is a calendar of the year: each day's colour is the time
  played, its number the Steam achievements unlocked. Under it, the month day by day, split by
  game. Mantel writes every session down from wsq.9 on, and reads past Steam sessions from the
  Steam client's own log (`Steam\logs\gameprocess_log.txt`) as far back as Steam keeps it;
  achievement dates go back further.
- **Play history and HowLongToBeat.** A game's page lists every day it was played, with the
  breaks between. Link the game to its HowLongToBeat page once and Home and the page show how
  much of the story is left.
- **HDR for the games that want it.** A game's page turns HDR on for that game: Mantel
  switches it on before the game starts and back off when it ends, through Windows or the
  NVIDIA driver (Settings → System). With NVIDIA, the same button also offers **RTX HDR**:
  the driver turns the game's SDR picture into HDR. Mantel sets it in the driver's profile for
  that game only while it runs, and puts the profile back afterwards, after a crash too.
- **Steam in Big Picture.** Optionally, Steam games start with Steam in Big Picture, which
  closes again after the game.
- **Tray icon.** Optionally, minimizing or closing hides Mantel in the notification area.
- **Trailers.** Rest on a game for a moment and its trailer plays, muted, behind it.
  Settings → Interface turns that off.
- **Apps next to the games.** The Apps tab opens music, video, chat and browsers: Microsoft
  Store and Edge are there from the start, and Spotify, Netflix, Discord and other apps you
  have join by themselves. Anything else adds from the Start menu, a list of sites such as
  YouTube and Twitch that open in their own window, or a file. Apps open, switch and close
  like games, but an open app is only a dot on its tile: no timer, no "Playing", and it
  doesn't hold back updates. Time in apps isn't counted as playtime.
- **Steps aside while you play.** Mantel minimizes when a game starts and comes back when
  it exits.
- **Art for everything.** Covers, backgrounds and logos come from Steam and GOG; a free
  SteamGridDB key covers the rest.

## Install

Download `Mantel_x.y.z_x64-setup.exe` from
[Releases](https://github.com/wooshaq/mantel/releases) and run it. On first start, a short
setup asks for your language, signs you in to Steam and Xbox if you like, and can make Mantel
start with Windows. Settings → System →
*Run setup again* brings it back. This build (wooshaq) has no auto-updater (it never downloads or installs anything by itself) and
never sets itself as the home app of Xbox mode: no package registration, no Developer Mode. It
runs no PowerShell either: Store packages and Start menu entries are read through Windows' own APIs.

The installer isn't code-signed yet, so Windows may show "Windows protected your PC":
choose *More info → Run anyway*. Signing will come through SignPath Foundation; see the
[code signing policy](https://playmantel.com/code-signing).

## Where games come from

| Store | Installed games |
| --- | --- |
| Steam | Library folders and app manifests, playtime from Steam's own files; achievements from the public profile |
| Xbox / Game Pass / Microsoft Store | Packages that ship a `MicrosoftGame.config` |
| Epic Games | Launcher manifests and catalog cache |
| GOG | Registry; launches through Galaxy when installed |
| Ubisoft Connect | Registry |
| Anything else | Settings → *Add a game manually*: suggested shortcuts, or any `.exe`, `.lnk` or `.url` |

## Privacy

Mantel has no servers, sends no telemetry and signs in to no account. It talks only to:

- **Steam's public Web API and community pages**, for achievements of the Steam games you have
  played: it asks for the public profile of the account Steam last signed in with on this PC
  (read from Steam's `loginusers.vdf`), or the profile you enter. Nothing is sent but the
  SteamID and the game's app id.
- **HowLongToBeat**, only for games you link to it: Mantel reads the public page of each linked
  game, about every two weeks. HowLongToBeat has no API and blocks apps from its search, so
  Mantel never searches it. Linking opens HowLongToBeat's own site in a window over Mantel,
  already searching for the game: you pick the game, and Mantel takes the address of its page.
- **Store catalogs.** Microsoft's public Game Pass and Store catalogs and Steam's store, for
  the lists to browse and for search, which sends them what you type. No account is involved
  in reading them.
- **Installing.** Store games open in the Xbox app, which installs them under the Microsoft
  account signed in to Windows; Steam installs go through the Steam client.
- **Artwork and trailers.** Steam's store, image and video servers, GOG's public API and,
  if you add a key, SteamGridDB.

Everything stays on your PC in `%APPDATA%\com.playmantel.launcher\`: `user.json` holds
settings, favorites and playtime; `sessions.json` the log of sessions the statistics are drawn
from; `cache.json` can be deleted at any time. Sign-ins an older
version saved are deleted from `user.json` on the first start of this one, along with a Steam
session in Mantel's WebView.

## Controls

| Button | Action |
| --- | --- |
| A | Play |
| B | Back |
| Y | Game details |
| X | Favorite |
| LB / RB | Games ↔ Library ↔ Xbox ↔ Steam ↔ Settings |
| LT / RT | Filters |
| View (⧉) | Search |
| Menu (≡) | Control center: volume, brightness, close game, restart, power off, desktop |

On a keyboard: arrows, Enter, Esc, `i` details, `f` favorite, `q` / `e` tabs, `/` search,
`m` menu.

## Develop

Mantel is [Tauri 2](https://tauri.app) (Rust) with a React + TypeScript UI.

```bash
npm install
npm run dev          # the UI in a browser with a sample library, at http://localhost:1420
```

A gamepad connected to your computer works in the browser too. On Windows,
`npm run tauri dev` runs the real app and `npm run tauri build` makes the installer.

### From a Mac

The Windows exe cross-compiles with [cargo-xwin](https://github.com/rust-cross/cargo-xwin)
(`brew install llvm lld`, `rustup target add x86_64-pc-windows-msvc`,
`cargo install cargo-xwin`):

```bash
PATH="/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/lld/bin:$PATH" npm run build:win
```

This builds the portable `src-tauri/target/x86_64-pc-windows-msvc/release/mantel.exe`;
installers come from CI.

### Releasing

1. Bump `version` in `package.json` (the app reads its version from there).
2. Push a tag: `git tag v0.2.0 && git push origin v0.2.0`.
3. CI builds the installer into a draft release.

Updates are signed: the release job needs the `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repository secrets, matching `plugins.updater.pubkey`
in `src-tauri/tauri.conf.json`.

### Forks

This fork has no sign-ins, no updater and no Xbox app registration.

## Contributing

Issues and pull requests are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md).

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by
[SignPath Foundation](https://signpath.org). Only installers that GitHub Actions builds from
this repository are signed, and each signing request is approved by hand. Team roles, what the
installer changes and the privacy notes are on [playmantel.com/code-signing](https://playmantel.com/code-signing).

## License

[GPL-3.0-or-later](LICENSE). Mantel is not affiliated with Valve, Microsoft, ASUS, Epic
Games, GOG or Ubisoft. Game names and artwork belong to their owners.
