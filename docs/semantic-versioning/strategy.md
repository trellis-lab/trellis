# Semantic Versioning Strategy

## Goals

- Single version number across the entire Trellis workspace (all Rust crates, VS Code extension, IntelliJ plugin, Docker image).
- Version derived automatically from git history and Conventional Commit messages — no manual version bumps in files.
- Pre-release builds on `develop` and `release/*` branches get distinct, non-ambiguous version labels.
- Release pipeline remains tag-triggered; tagging is automated on merge to `main`.

---

## Tool: GitVersion

[GitVersion](https://gitversion.net) reads git history and branch names to produce a deterministic SemVer string on every commit. It runs inside GitHub Actions with zero local tooling required.

**Mode: `ContinuousDelivery`** — version is stable on `main` and release branches; pre-release suffix is appended everywhere else. This is the canonical mode for Gitflow.

---

## Branch → Version Mapping

| Branch | Example version | Label | Trigger |
|---|---|---|---|
| `main` | `1.2.3` | none | merge from `release/*` or `hotfix/*` |
| `release/1.2.0` | `1.2.0-preview.4` | `preview` | commit to release branch |
| `develop` | `1.3.0-dev.17` | `dev` | any commit |
| `feature/*` | `1.3.0-dev.5` | `dev` | any commit |
| `hotfix/*` | `1.2.4` | none | merge to `main` |
| `cycle/*` | `1.3.0-dev.3` | `dev` | any commit |

The numeric suffix (`.17`, `.5`) is the commit distance from the last version tag — it increments monotonically and makes every build uniquely addressable.

---

## Conventional Commits → Bump Rules

GitVersion reads commit messages to decide whether the next release increments major, minor, or patch.

| Commit prefix | Bump | Example |
|---|---|---|
| `feat(scope):` | **minor** | `feat(routing): add hop arc rendering` |
| `fix(scope):` | patch | `fix(labels): prevent overlap on narrow nodes` |
| `refactor(scope):` | patch | `refactor(ports): extract common helpers` |
| `perf(scope):` | patch | `perf(routing): cache grid cell lookup` |
| `docs(scope):` | patch | `docs(api): update WASM binding docs` |
| `chore(scope):` | patch | `chore(deps): bump resvg to 0.44` |
| `test(scope):` | patch | `test(er): add crow-foot fixture` |
| `build(scope):` | patch | `build(ci): add IntelliJ plugin step` |
| any `!` suffix | **major** | `feat(ast)!: rename NodeShape variants` |
| `BREAKING CHANGE:` footer | **major** | (footer in commit body) |

> **Rule**: scope is optional but recommended. Commits without a recognized prefix do not bump the version — use this for merge commits, CI fixes, and housekeeping that should not affect the version.

---

## GitVersion Configuration

Place this file at the repository root as `GitVersion.yml`.

```yaml
# GitVersion.yml
mode: ContinuousDelivery
tag-prefix: 'v'
assembly-versioning-scheme: MajorMinorPatch

major-version-bump-message: "^(feat|fix|refactor|perf|chore|docs|style|test|build|ci)(\\(.+\\))?!:|^BREAKING CHANGE"
minor-version-bump-message: "^feat(\\(.+\\))?:"
patch-version-bump-message: "^(fix|refactor|perf|docs|style|chore|test|build|ci)(\\(.+\\))?:"
no-bump-message: "^(chore|docs)(\\(.+\\))?:\\s*(no-bump|skip-bump)"

branches:
  main:
    regex: ^main$
    tag: ''
    increment: Patch
    prevent-increment-of-merged-branch-version: true
    track-merge-target: false

  develop:
    regex: ^develop$
    tag: dev
    increment: Minor
    track-merge-target: false

  release:
    regex: ^release[/-]
    tag: preview
    increment: Patch
    prevent-increment-of-merged-branch-version: true
    track-merge-target: true

  feature:
    regex: ^feature[/-]
    tag: dev
    increment: Inherit

  hotfix:
    regex: ^hotfix[/-]
    tag: ''
    increment: Patch
    prevent-increment-of-merged-branch-version: true

  cycle:
    regex: ^cycle[/-]
    tag: dev
    increment: Inherit

ignore:
  sha: []
```

### Key points

- **`fetch-depth: 0` is mandatory** in every `actions/checkout` step that precedes GitVersion. A shallow clone produces wrong or broken versions because GitVersion walks the full tag history.
- The `release/*` branch name encodes the target version (e.g., `release/1.2.0`). GitVersion reads this and uses it as the floor for version calculation on that branch.
- `track-merge-target: true` on release branches means the version will advance toward the release target even before the merge.

---

## Artifact Version Stamping

GitVersion outputs a JSON object with many fields. The ones used in this project:

| Field | Use |
|---|---|
| `SemVer` | Full version string, e.g., `1.2.0-preview.4` |
| `MajorMinorPatch` | Short stable version, e.g., `1.2.0` |
| `NuGetVersionV2` | Sanitized pre-release label (safe for package registries) |
| `InformationalVersion` | Full version + git SHA suffix |

### Rust / Cargo

`Cargo.toml` workspace version is updated in CI before the build using `cargo-set-version` (part of `cargo-edit`):

```bash
cargo install cargo-edit --locked
cargo set-version "$SEMVER"
```

All crates inherit via `[workspace.package]` version — single update propagates everywhere.

### VS Code Extension

`package.json` version is updated with `npm`:

```bash
npm version "$SEMVER" --no-git-tag-version
```

`--no-git-tag-version` prevents npm from creating a conflicting git tag.

### IntelliJ Plugin

The version is injected via a Gradle property:

```bash
# In the workflow step:
./gradlew buildPlugin -Pplugin.version="$SEMVER" -x buildSearchableOptions
```

`build.gradle.kts` reads it as:

```kotlin
version = providers.gradleProperty("plugin.version").getOrElse("0.0.0-dev")
```

### Docker Image

Tags derived from the computed version:

```yaml
tags: |
  ghcr.io/${{ github.repository }}:${{ env.SEMVER }}
  ghcr.io/${{ github.repository }}:latest        # only on main
  ghcr.io/${{ github.repository }}:preview        # only on release/*
```

---

## Pipeline Architecture

### New: `versioning` Reusable Job

Added to both `ci.yml` and `release.yml` as the first job. All downstream jobs receive the version string via job outputs.

```yaml
versioning:
  name: Compute Version
  runs-on: ubuntu-latest
  outputs:
    semver: ${{ steps.gv.outputs.semVer }}
    short: ${{ steps.gv.outputs.majorMinorPatch }}
    pre-release-tag: ${{ steps.gv.outputs.preReleaseTag }}
    informational: ${{ steps.gv.outputs.informationalVersion }}
  steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0        # REQUIRED — full history for GitVersion
    - uses: gittools/actions/gitversion/setup@v3
      with:
        versionSpec: '6.x'
    - id: gv
      uses: gittools/actions/gitversion/execute@v3
```

All subsequent jobs declare `needs: versioning` and consume `needs.versioning.outputs.semver`.

### New: `auto-tag.yml` Workflow

Runs on push to `main`. Computes version with GitVersion, creates and pushes a `v{semver}` tag, which in turn triggers the existing `release.yml`.

```yaml
name: Auto Tag

on:
  push:
    branches: [main]

permissions:
  contents: write

jobs:
  tag:
    name: Create Version Tag
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
          token: ${{ secrets.GITHUB_TOKEN }}

      - uses: gittools/actions/gitversion/setup@v3
        with:
          versionSpec: '6.x'

      - id: gv
        uses: gittools/actions/gitversion/execute@v3

      - name: Create and push tag
        env:
          SEMVER: ${{ steps.gv.outputs.semVer }}
        run: |
          git config user.name  "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git tag "v${SEMVER}" -m "Release v${SEMVER}"
          git push origin "v${SEMVER}"
```

> **Guard**: if the tag already exists (e.g., re-running CI on the same commit), the push is a no-op and does not fail the workflow — add `|| true` to the push line if needed.

### Updated `ci.yml`

Changes to existing CI:

1. Add `versioning` job (runs on all branches).
2. Add `fetch-depth: 0` to all `checkout` steps.
3. Pass `needs.versioning.outputs.semver` as env var to build jobs (for artifact naming, not stamping — stamping happens only in `release.yml`).

### Updated `release.yml`

Changes to existing release pipeline:

1. Add `versioning` job as first dependency.
2. Add version stamping step in each build job before the build command.
3. Replace `${{ github.ref_name }}` with `${{ needs.versioning.outputs.semver }}` throughout.
4. Update Docker tags to use computed version + branch-conditional `latest`/`preview` tags.

---

## Release Process (Step by Step)

### Normal feature release (minor bump)

```
feature/my-feature  →  develop  →  release/1.3.0  →  main
                                        ↑ preview builds       ↑ auto-tag v1.3.0
```

1. Developer opens PR from `feature/my-feature` → `develop`.
   - Commit message: `feat(routing): add hop arc rendering`
   - CI computes version: `1.3.0-dev.N`
2. Merge to `develop`. CI builds dev artifact `1.3.0-dev.N`.
3. When ready to ship: create branch `release/1.3.0` from `develop`.
   - CI immediately produces `1.3.0-preview.1`.
   - Only bug fixes merged into `release/1.3.0` during stabilization.
   - Each fix commit: `fix(labels): correct offset calculation` → `1.3.0-preview.2`, etc.
4. Merge `release/1.3.0` → `main` via PR.
   - `auto-tag.yml` fires → creates tag `v1.3.0`.
   - `release.yml` fires on that tag → builds and publishes all artifacts.
5. Merge `release/1.3.0` → `develop` (backport). **Required** to keep develop in sync.

### Patch / hotfix release

```
hotfix/fix-crash  →  main
                       ↑ auto-tag v1.3.1
```

1. Create `hotfix/fix-crash` from `main`.
2. Commit: `fix(pipeline): prevent panic on empty graph`
3. Merge `hotfix/fix-crash` → `main`.
   - `auto-tag.yml` → `v1.3.1`.
   - `release.yml` → publishes.
4. Merge `hotfix/fix-crash` → `develop` too.

### Breaking change (major bump)

Same as normal feature release but commit message carries `!`:

```
feat(ast)!: rename NodeShape variants
```

GitVersion reads the `!` and bumps major. Create `release/2.0.0` branch accordingly.

---

## Migration Plan

Current state: workspace at `0.1.0`, release triggered manually by pushing `v*` tags.

| Step | Action |
|---|---|
| 1 | Add `GitVersion.yml` to repo root |
| 2 | Tag current `main` HEAD as `v0.1.0` (establishes baseline for GitVersion) |
| 3 | Update `Cargo.toml` `[workspace.package].version` to `0.0.0` — CI will overwrite it; this avoids stale committed version |
| 4 | Add `versioning` job to `ci.yml`; add `fetch-depth: 0` to all checkouts |
| 5 | Add `auto-tag.yml` workflow |
| 6 | Update `release.yml`: add `versioning` job, add stamping steps, replace `github.ref_name` refs |
| 7 | Update `build.gradle.kts` to read `plugin.version` Gradle property |
| 8 | Test on a `release/0.2.0` branch — verify preview label appears, verify merge to main auto-tags |

> **Step 3 note**: Committing `0.0.0` as the workspace version is intentional. It signals "this file is not the source of truth." The real version lives in git tags and is injected at build time. A pre-commit hook or CI lint step can enforce that the committed value stays `0.0.0`.

---

## Artifact Matrix

| Artifact | Versioned by | Published where | When |
|---|---|---|---|
| `trellis` CLI binaries | Cargo (stamped in CI) | GitHub Release assets | on `v*` tag |
| `trellis-*.vsix` | `package.json` (stamped in CI) | GitHub Release assets (manual marketplace later) | on `v*` tag |
| `trellis-intellij-*.zip` | Gradle property (injected in CI) | GitHub Release assets (manual marketplace later) | on `v*` tag |
| `ghcr.io/.../trellis:*` | Docker tag | GitHub Container Registry | on `v*` tag |
| Rust crates | Cargo (stamped in CI) | Not published (internal only) | — |

---

## Version Visibility in Artifacts

- **CLI**: `trellis --version` outputs `trellis 1.3.0` (from `CARGO_PKG_VERSION`, set by stamped `Cargo.toml`).
- **VS Code extension**: Version visible in Extensions panel and `package.json`.
- **IntelliJ plugin**: Version in `plugin.xml` and displayed in IDE plugin list.
- **Docker image**: Tag on GHCR; `docker inspect` shows label `org.opencontainers.image.version`.

Add OCI labels to the Docker build step:

```yaml
labels: |
  org.opencontainers.image.version=${{ needs.versioning.outputs.semver }}
  org.opencontainers.image.revision=${{ github.sha }}
  org.opencontainers.image.source=${{ github.server_url }}/${{ github.repository }}
```

---

## Open Questions / Deferred

- **crates.io publishing**: Not in scope. If added later, `cargo publish` uses the stamped `Cargo.toml` version — no extra work required.
- **VS Code Marketplace auto-publish**: Deferred. When ready, add `vsce publish` step gated on `github.ref_name` matching `v[0-9]*` and a `VSCE_PAT` secret.
- **JetBrains Marketplace auto-publish**: Deferred. Gradle IntelliJ Plugin provides `publishPlugin` task; needs `JETBRAINS_TOKEN` secret.
- **CHANGELOG generation**: `release.yml` already uses `generate_release_notes: true` on the GitHub Release — this covers the basic case. If a structured `CHANGELOG.md` is needed, add `conventional-changelog-cli` as a step after tagging.
