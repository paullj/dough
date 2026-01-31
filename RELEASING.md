# Release Process

This project uses automated releases with `release-plz` and `cargo-dist`.

## How it works

1. **Automatic PR Creation**: When you push to `main`, release-plz analyzes commits and creates a PR if there are releasable changes
2. **Version Bumping**: The PR updates version in `Cargo.toml` and generates/updates `CHANGELOG.md`
3. **Tag Creation**: When you merge the release PR, release-plz creates a git tag (e.g., `v0.1.1`)
4. **Binary Building**: The tag triggers cargo-dist which builds binaries for:
   - macOS (Intel & Apple Silicon)
   - Linux (x86_64 & aarch64)
   - Windows (x86_64)
5. **GitHub Release**: cargo-dist creates a GitHub release with all binaries attached

## Configuration Files

- `release-plz.toml` - Configures version bumping and PR creation
- `dist-workspace.toml` - Configures binary building and distribution
- `.github/workflows/release-plz.yml` - Triggers on push to main
- `.github/workflows/release.yml` - Triggers on tag creation

## CI Behavior

- Regular CI (tests/builds) skip release commits (those with "chore: release" in message)
- This prevents redundant builds when release-plz creates commits

## Manual Release

If needed, you can trigger a release manually:
```bash
# Create and push a version tag
git tag v0.1.1
git push origin v0.1.1
```

This will trigger cargo-dist to build and release binaries.