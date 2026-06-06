# Distribution Plan

Koba is distributed for Windows through a custom Scoop bucket and through GitHub Release artifacts. Other package-manager channels remain intentionally manual for now.

## Recommended Phases

### Phase 0: Local Install

Local development install path:

```sh
cargo install --path crates/koba
```

This requires Rust and installs from the checked-out workspace.

### Phase 1: Local Release Artifact Builds

Use `dist`/cargo-dist for local artifact planning and binary archive builds.

Initial repo config is intentionally conservative:

- `profile.dist` builds from release with thin LTO.
- `workspace.metadata.dist.packages = ["koba"]`.
- Desktop targets are listed for Linux, macOS Intel, macOS Apple Silicon, and Windows.
- `installers = []` so no shell, npm, Homebrew, MSI, or other installer is generated yet.
- `ci = "github"` and `hosting = "github"` enable GitHub Release artifact builds from tags.
- No package-manager publishing jobs, secrets, taps, buckets, or registries are configured.

Recommended local review commands once `dist` is installed:

```sh
dist plan
dist build --artifacts=local --target=<current-target> --output-format=json
```

Use `rustc -vV` to confirm the current host target. On Windows, the validated command was:

```sh
dist build --artifacts=local --target=x86_64-pc-windows-msvc --output-format=json
```

Review `target/distrib/` after the build for archives, checksums, and the unpacked binary.

### Phase 2: GitHub Releases

Use GitHub Releases as the first public binary channel.

Koba has a tag-driven release workflow at `.github/workflows/v-release.yml`.

The workflow:

- runs only for pushed tags matching `v*`;
- installs cargo-dist `0.32.0`;
- asks cargo-dist to plan the release;
- builds archives for configured desktop targets;
- generates checksums and a source archive;
- creates a GitHub Release and uploads the artifacts with `GITHUB_TOKEN`.

Configured artifact targets:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Recommended release path:

1. Keep CI green.
2. Review `dist plan` output locally.
3. Build at least one local artifact with `dist build --artifacts=local`.
4. Update the crate version and changelog.
5. Commit the release prep through a normal PR.
6. Create and push a version tag only when ready:

```sh
git tag v0.1.0
git push origin v0.1.0
```

Pushing the tag triggers the release workflow. Do not create tags until the release commit is reviewed and CI is green.

### Phase 3: Install Script

After GitHub Release artifacts exist, add shell and PowerShell install scripts. These should download pinned release assets and verify checksums.

Do not ship install scripts before release artifacts and checksum behavior are stable.

### Phase 4: Homebrew Tap

Homebrew should come after GitHub Releases. A dedicated tap such as `postigodev/homebrew-tap` can publish formulas that point at release assets.

Do not configure a tap until the repository exists, token permissions are understood, and release artifacts are stable.

### Phase 5: Scoop Bucket

Implemented and validated for Windows through the custom bucket:

```powershell
scoop bucket add postigodev https://github.com/postigodev/scoop-bucket
scoop install koba
```

The bucket manifest downloads the cargo-dist Windows archive:

```text
koba-x86_64-pc-windows-msvc.zip
```

The archive contains `koba.exe` at its root, and Scoop verifies the release SHA-256 before installing. The installed shim exposes `koba` globally.

Validated behavior:

- `scoop install koba` installed Koba `0.1.1`.
- `koba scan` and `koba doctor` worked from an unrelated repository.
- `scoop update koba` completed successfully.
- uninstall and reinstall were tested successfully.

Maintenance expectation:

- Keep `packaging/scoop/bucket/koba.json` as the reviewable source manifest in this repo.
- Publish the manifest to `postigodev/scoop-bucket` for end users.
- Let Scoop `checkver`/`autoupdate` track GitHub releases where possible.
- Manually review manifest updates before merging them into the bucket.

### Phase 6: npm/npx or pnpx Wrapper

An npm wrapper can make `npx koba` or `pnpx koba` convenient for JavaScript-heavy users, but it adds packaging complexity:

- Native binaries must be selected per platform.
- npm package names and scopes need ownership decisions.
- Postinstall download scripts have security and reliability tradeoffs.
- Publishing introduces a second release channel that must stay in sync.

Treat npm as a convenience wrapper over GitHub Release binaries, not as the canonical build source.

### Phase 7: winget

Winget should follow stable Windows release artifacts. It requires manifest updates and installer/download URLs that should not churn.

Keep winget manual until release cadence and checksums are predictable.

### Phase 8: apt/deb

Debian packages are useful later, especially for Linux users who prefer system package managers. They add signing, repository hosting, dependency metadata, and upgrade semantics.

Do not prioritize apt/deb until direct GitHub Release installs and Homebrew/Scoop are working.

## What Not To Automate Yet

- crates.io publishing.
- npm publishing.
- Homebrew tap publishing.
- Scoop publishing beyond the custom `postigodev/scoop-bucket` bucket.
- winget submission.
- apt/deb repository publishing.
- Tag creation.
- Release signing or Windows code signing.
- Any workflow requiring repository secrets.

## Recommended First Release Path

1. Keep local Cargo install available for development.
2. Validate dist locally with `dist plan`.
3. Build one local artifact with `dist build --artifacts=local`.
4. Review artifact names, archive contents, and checksums.
5. Cut GitHub Releases by pushing reviewed `v*` tags.
6. Update or verify the Scoop manifest for each release.
