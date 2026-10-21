# How Labyrica apps ship

One way, for every app. If an app follows this page, Labyrica Dispatch can
install, update and uninstall it with a two-line catalog entry.

## The rule

Push a tag `vX.Y.Z` on the app's repo. A signed, verified build appears in
the public repo `Labyrica/<id>-releases`. That's the whole release process.

```sh
# bump the version (package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json)
git commit -am "v0.4.0" && git tag v0.4.0 && git push --follow-tags
```

## Per app, once

| What | Convention |
|---|---|
| Id | lowercase, dashes: `hightower`, `foundry-godot`. Used everywhere below. |
| Source repo | `Labyrica/<anything>`, private or public. |
| Downloads repo | `Labyrica/<id>-releases`, public, holds only releases. |
| Tags | `vX.Y.Z`. Nothing else triggers a release. |
| Release workflow | `.github/workflows/release.yml` calling the shared one (below). |
| Signing key | `npx tauri signer generate -w ~/.tauri/<id>.key -p ""`, then secret `TAURI_SIGNING_PRIVATE_KEY` on the source repo. Public key goes in the Dispatch catalog. |
| Beacon | copy [`snippets/dispatch_beacon.rs`](snippets/dispatch_beacon.rs), call it from `.setup`. |

New Tauri apps also use, so the catalog needs no overrides:

| `tauri.conf.json` | Value |
|---|---|
| `identifier` | `com.labyrica.<id>` |
| `mainBinaryName` | `<id>` |
| `bundle.publisher` | `Labyrica` |
| `bundle.createUpdaterArtifacts` | `true` |
| `bundle.windows.nsis.installMode` | `currentUser` (no admin prompt) |
| `plugins.updater.endpoints` (optional) | `https://github.com/Labyrica/<id>-releases/releases/latest/download/latest.json` |

Older apps keep their existing identifiers (changing one would orphan users'
settings); the catalog lists those as overrides.

## Release workflows

Tauri apps:

```yaml
name: release
on:
  push:
    tags: ["v*"]
permissions:
  contents: write
jobs:
  release:
    uses: Labyrica/.github/.github/workflows/tauri-release.yml@v1
    with:
      app-name: Hightower
      releases-repo: Labyrica/hightower-releases
      # project-path: apps/desktop      # if src-tauri isn't at the root
      # package-manager: pnpm
      # prepare: node scripts/fetch-sidecar.mjs --target "$TARGET"
    secrets: inherit
```

Anything else (a Node bundle, a Swift app): `bundle-release.yml`, with a build
script that leaves its files in `./out/`. See the header of
[that workflow](.github/workflows/bundle-release.yml).

Both end in [`publish.yml`](.github/workflows/publish.yml):

1. **Verify.** The draft must contain every asset Dispatch expects, with
   signatures. If not, nothing is published.
2. **Mirror** to `<id>-releases`, rewriting `latest.json` to the public URLs.
3. **Finalize** the draft on the source repo.

## What Dispatch expects

| Kind | Windows | macOS |
|---|---|---|
| Tauri app | `*_x64-setup.exe` + `.sig` | `*_{aarch64,x64}.app.tar.gz` + `.sig`, `.dmg` for people |
| Bundle | `*_windows-x64.zip` + `.sig` | `*_macos-{aarch64,x64}.zip` + `.sig` |
| Mac app (Swift) | n/a | `*_{aarch64,x64}.app.tar.gz` + `.sig` |

## Org secrets

| Secret | Where | What |
|---|---|---|
| `RELEASES_PAT` | org | Fine-grained token, contents: read/write on `Labyrica/*-releases`. |
| `TAURI_SIGNING_PRIVATE_KEY` | each app repo | That app's key. `_PASSWORD` empty. |
| `APPLE_*` | org, optional | Developer ID signing + notarization. Without them Macs show "Open Anyway" once. |
