# devclean

A macOS menu-bar app and CLI that shows what is eating your memory and disk, and clears
developer caches safely. Rust core shared by a Tauri 2 app and a `devclean` command; one
universal build runs on Intel and Apple Silicon Macs (macOS 13+). Docker, pnpm, uv and the
rest are used only when installed.

Built by Zain Zafar.

```
devclean                          memory, disk and cache report (⚠ marks anything too high)
devclean clean safe               clean everything that rebuilds itself on demand
devclean clean <area>             clean one area
devclean clean <area> --dry-run   show what would be cleaned, change nothing
devclean schedule on|off|status   daily 03:00 clean of Docker leftovers
devclean version
```

Exit codes: 0 done, 1 something failed (the error is printed), 2 usage error.

## Menu-bar app

The menu bar shows free memory and free disk, e.g. `47% · 49.6 GB`, with a ⚠ in front when
memory is under 20% free or disk under 10% free. Left-click opens the panel: memory pressure,
swap and the six heaviest apps, disk space, every cache area with its size and a Clean button
(areas marked *ask* confirm first), **Clean safe**, the daily-clean switch and the last
scheduled run. It refreshes every 5 seconds while open; cache sizes fill in one by one as each
is measured. Esc closes it, ⌘R refreshes, ⌘Q quits; right-click the icon for About and Quit.
The panel uses Liquid Glass on macOS 26 and the popover material on older systems, follows the
system appearance and accent colour, and has no Dock icon.

## Areas

| Area | What it clears | Safe |
|---|---|---|
| docker | unused images, anonymous volumes, build cache, test containers over a day old, stopped buildx builders; keeps running containers and named volumes | yes |
| pnpm, uv, npm, yarn, pip | the package manager's own prune/clean command | yes |
| brew | `brew cleanup -s --prune=all` | yes |
| xcode | Xcode DerivedData | yes |
| simulators | unavailable simulators and simulator caches | yes |
| playwright | downloaded browsers (re-download with `npx playwright install`) | ask |
| cocoapods | CocoaPods cache | ask |
| worktrees | git worktrees of repos listed in `~/.config/devclean/config` that have no changes and no unpushed commits | ask |

Safety rules:

- Nothing outside `~/Library` and `~/.cache` is ever deleted directly. A path guard rejects
  anything else, `..`, the roots themselves, and symlinks whose real location escapes them.
- Everything else goes through the tool's own command (`docker … prune`, `pnpm store prune`,
  `git worktree remove`, which refuses dirty worktrees).
- Docker prunes only what is older than 24 hours, removes only stopped test containers, and removes
  volumes only when they are unused and anonymous (named volumes are never touched, on any Docker version).
- Worktrees are removed only when `git status` is clean and `HEAD` has no commits missing from
  a remote (after `git fetch origin`), re-checked right before each removal. Cleaning worktrees
  always asks first: the CLI lists each worktree it would remove, including git-ignored files such
  as `.env` that go with it, and removes nothing unless you answer `y`.
- Every external command has a timeout; failures are reported, never swallowed.

The daily clean is a LaunchAgent (`~/Library/LaunchAgents/com.devclean.daily.plist`) that runs
`devclean scheduled` at 03:00 and appends to `~/Library/Logs/devclean.log`. It cleans Docker
leftovers only.

## Install

```
./install.sh                          # universal CLI into ~/.local/bin, daily clean on
DEVCLEAN_NO_SCHEDULE=1 ./install.sh   # without the daily clean
```

App: build it (below) and copy `target/universal-apple-darwin/release/bundle/macos/devclean.app`
to `/Applications`. The build is unsigned, so the first launch needs right-click → Open (or
`xattr -dr com.apple.quarantine /Applications/devclean.app`). To start it at login, add it
under System Settings → General → Login Items. The app's daily-clean switch registers
`~/.local/bin/devclean` when the CLI is installed, otherwise the app binary itself.

## Develop

```
crates/devclean-core   library: memory, processes, disk, areas, sizing, clean, path guard, schedule
crates/devclean-cli    the devclean binary
app/                   React + Vite + Tailwind + shadcn/ui panel (features/ per card)
app/src-tauri          Tauri 2 shell: tray, panel window, commands over devclean-core
```

```
cargo test --workspace                          # needs app/dist: run `pnpm build` in app/ once
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cd app && pnpm install && pnpm lint && pnpm typecheck && pnpm test
cd app && pnpm tauri dev                        # run the app with hot reload
```

Tests never touch real caches: cleaning is exercised under temporary home directories and
temporary git repositories.

## Build

```
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cd app && pnpm tauri build --target universal-apple-darwin   # or: cargo tauri build …
scripts/build-cli-universal.sh                               # target/cli-universal/devclean
```

The app bundle lands in `target/universal-apple-darwin/release/bundle/macos/devclean.app`.
Icons are generated from `app/icon-source/app-icon.svg` (`pnpm tauri icon icon-source/app-icon.png
-o src-tauri/icons`); the menu-bar icon is the template image `src-tauri/icons/tray-icon@2x.png`.

### Signing and notarization (later)

1. Get a *Developer ID Application* certificate and install it in the login keychain.
2. Set `bundle.macOS.signingIdentity` in `app/src-tauri/tauri.conf.json` (or export
   `APPLE_SIGNING_IDENTITY`), then build; Tauri signs with the hardened runtime.
3. Notarize during the build by exporting `APPLE_ID`, `APPLE_PASSWORD` (app-specific password)
   and `APPLE_TEAM_ID`, or afterwards with
   `xcrun notarytool submit devclean.dmg --keychain-profile <profile> --wait` and
   `xcrun stapler staple devclean.app`.
4. Sign the CLI too: `codesign --force --options runtime --timestamp -s "Developer ID Application: …" target/cli-universal/devclean`.
