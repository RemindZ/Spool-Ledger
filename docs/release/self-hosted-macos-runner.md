# Self-hosted macOS release runner

The registered Apple Silicon MacBook is a release appliance, not a general CI worker. Pull requests and ordinary pushes must never route jobs to it.

## One-time host setup

1. Create a dedicated non-admin local account for GitHub Actions.
2. Keep personal keychains, SSH keys, iCloud folders, browser profiles, and unrelated repositories unavailable to that account.
3. Install current macOS updates and Xcode Command Line Tools.
4. Accept the Xcode license and confirm `xcode-select -p` succeeds.
5. Install Rosetta 2 if the universal toolchain requires an Intel helper.
6. Register the runner for this repository only.
7. Add the custom label `spool-ledger-release`. The complete required label set is `self-hosted`, `macOS`, `ARM64`, and `spool-ledger-release`.
8. Configure the runner service to start at login and verify it appears online before tagging a release.

No Apple signing certificate, Apple ID, notarization password, or Windows signing secret belongs on this runner while releases are unsigned.

## Repository controls

- Protect `main` and tags matching `v*` with GitHub rulesets.
- Restrict tag creation to the repository owner.
- Require owner review for `.github/workflows/`, release scripts, compatibility evidence, and the release matrix through `CODEOWNERS`.
- Keep every pull-request workflow on GitHub-hosted runners.
- Permit only the reviewed, immutable action revisions present in the workflows.

A custom runner label is routing, not a security boundary. Workflow ownership and protected tags prevent unreviewed repository code from selecting the MacBook.

## Release behavior

The MacBook receives only the protected-tag `build-macos` release job and the manually dispatched acceptance job from protected `main`. The acceptance exception exists so a universal build can be proven before its fail-closed release-matrix row is enabled. Neither path accepts pull-request or ordinary-push code. These jobs:

- checks the host architecture and Xcode tools;
- checks out the immutable commit from a protected release tag or protected `main`;
- installs locked Node and Rust dependencies;
- runs frontend and Rust verification;
- builds one universal application and DMG;
- proves the executable contains `arm64` and `x86_64`;
- uploads short-lived workflow artifacts;
- removes build output and dependencies from its Actions workspace even after failure.

If the MacBook is offline, the job waits until its timeout. Bring the runner online and use **Re-run failed jobs**. The draft release assembler cannot run without a successful macOS result.

## Periodic maintenance

- Apply macOS, Xcode tools, GitHub runner, Rust, and Node security updates.
- Confirm free disk space before release tags.
- Review runner diagnostics after interrupted jobs.
- Remove stale Actions workspaces without touching files outside the dedicated runner account.
- Re-register the runner if its repository token or machine ownership changes.
