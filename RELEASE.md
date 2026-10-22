# Release Process

This document describes the process for releasing new versions of Bnpm.

## Versioning Scheme

Bnpm follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(`MAJOR.MINOR.PATCH`):

- **MAJOR** — Breaking changes (incompatible API changes)
- **MINOR** — New features (backward compatible)
- **PATCH** — Bug fixes (backward compatible)

We use [Changesets](https://github.com/changesets/changesets) to manage
versioning and changelog generation. Each PR that introduces a change must
include a changeset describing the change and its semver impact.

## Release Cadence

| Channel | Frequency | Purpose |
|---------|-----------|---------|
| `latest` | Every 2 weeks | Stable releases |
| `next` | Every PR merge | Preview of in-progress work |
| `alpha` | As needed | Experimental features |
| `beta` | Before major releases | Release candidates |

## Release Process (Standard)

### Step 1: Create Changeset

Each pull request that introduces a user-facing change must include a changeset:

```bash
bun changeset
```

Follow the prompts to:
1. Select the packages that have changed.
2. Choose the semver bump type (`patch`, `minor`, `major`).
3. Write a summary of the change.

Commit the generated `.md` file and include it in your PR.

### Step 2: Merge Changeset PR

Changesets accumulate on the `main` branch. When ready to release:

1. Create a "Version Packages" PR:
   ```bash
   bun changeset version
   ```

   This will:
   - Consume all unreleased changesets
   - Bump package versions accordingly
   - Update `CHANGELOG.md` files
   - Update any dependent packages

2. Review the Version Packages PR to ensure version bumps and changelogs
   look correct.

3. Merge the Version Packages PR.

### Step 3: CI Publishes to npm

When the Version Packages PR is merged to `main`, CI automatically:

1. Runs all checks (lint, test, build, typecheck).
2. Publishes updated packages to npm.
3. Publishes the Rust crate to crates.io.

### Step 4: Git Tag Is Created

CI creates a Git tag for each published package:

```
@better-npm/cli@0.2.0
bnpm-core@0.1.3
```

### Step 5: GitHub Release Is Created

CI creates a [GitHub Release](https://github.com/bnpm-io/bnpm/releases) with:

- Auto-generated release notes from the changelog
- Links to the relevant commits and PRs
- Binary artifacts attached (if applicable)

### Step 6: Docker Images Are Built and Published

CI builds and pushes Docker images:

```
ghcr.io/bnpm-io/api:0.2.0
ghcr.io/bnpm-io/web:0.2.0
ghcr.io/bnpm-io/cli:0.2.0
```

Each image is also tagged with `latest`.

### Step 7: Homebrew Formula Is Updated

CI updates the Homebrew formula in the
[bnpm-homebrew](https://github.com/bnpm-io/homebrew-bnpm) tap with the new
binary checksums.

## Hotfix Process

For critical bugs that cannot wait for the regular release cycle:

1. Create a `hotfix/` branch from the latest release tag:
   ```bash
   git checkout -b hotfix/critical-fix v0.2.0
   ```
2. Apply the fix and bump the patch version:
   ```bash
   bun changeset
   # Select patch bump
   ```
3. Open a PR targeting the `main` branch with the `hotfix` label.
4. Skip the Version Packages PR process — use the manual release command:
   ```bash
   bun changeset version --snapshot hotfix
   ```
5. Merge and publish as usual.

## Breaking Changes Policy

- Breaking changes must be clearly documented in the changelog.
- Breaking changes require:
  - A `semver:major` label on the PR.
  - A migration guide in the PR description.
  - Deprecation warnings for at least one minor release (when possible).
- Major version releases will include:
  - A comprehensive migration guide.
  - A blog post summarizing the changes.
  - At least one beta release before the stable major release.

## Pre-Release Channels

### Alpha (`alpha`)

For experimental features that are not yet stable:

```bash
bun changeset version --snapshot alpha
```

Published with the `alpha` dist-tag on npm.

### Beta (`beta`)

For release candidates before a major or minor release:

```bash
bun changeset version --snapshot beta
```

Published with the `beta` dist-tag on npm.

### Next (`next`)

For every PR merge (automated by CI):

```bash
bun changeset version --snapshot next
```

Published with the `next` dist-tag on npm.

### Installing Pre-Release Versions

```bash
npm install @better-npm/cli@next
# or
bnpm add @better-npm/cli@beta
```

## Manual Release (if CI is unavailable)

In case CI is down, releases can be performed manually:

```bash
# 1. Ensure you're on main and up to date
git checkout main && git pull

# 2. Version the packages
bun changeset version

# 3. Commit the version bump
git add .
git commit -m "chore: version packages"

# 4. Build all packages
bun run build

# 5. Publish to npm
bun changeset publish

# 6. Tag and push
git push --follow-tags
```
