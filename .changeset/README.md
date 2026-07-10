# Changesets

This directory manages versioning and changelogs for the Dash monorepo.

## How to add a changeset

Run `bun changeset` and follow the prompts. This will create a markdown file
in `.changeset/` that describes the change. Commit the file along with your
code changes.

## How to release

Changesets will automatically open a "Version Packages" PR that bumps versions
and updates changelogs. Merging that PR publishes to npm.
