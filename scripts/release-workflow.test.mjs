import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const root = resolve(import.meta.dirname, "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");

describe("release workflow policy", () => {
  it("uses the required tag-only release graph", () => {
    const workflow = read(".github/workflows/release.yml");
    expect(workflow).toContain('tags: ["v*"]');
    expect(workflow).not.toContain("pull_request:");
    expect(workflow).toMatch(/^ {2}preflight:/m);
    expect(workflow).toMatch(/^ {2}build-windows:/m);
    expect(workflow).toMatch(/^ {2}build-macos:/m);
    expect(workflow).toMatch(/^ {2}assemble-release:/m);
    expect(workflow).toMatch(/^ {2}attest-release:/m);
    expect(workflow).toMatch(/^ {2}assemble-draft-release:/m);
    expect(workflow).toMatch(/build-windows:[\s\S]*?needs: preflight/);
    expect(workflow).toMatch(/build-macos:[\s\S]*?needs: preflight/);
    expect(workflow).toMatch(
      /assemble-release:[\s\S]*?needs: \[preflight, build-windows, build-macos\]/,
    );
    expect(workflow).toMatch(/attest-release:[\s\S]*?needs: assemble-release/);
    expect(workflow).toMatch(
      /assemble-draft-release:[\s\S]*?needs: \[preflight, assemble-release, attest-release\]/,
    );

    const validator = read("scripts/validate-release.mjs");
    expect(validator).toContain(
      "needs: [preflight, build-windows, build-macos]",
    );
    expect(validator).not.toContain('"needs: [build-windows, build-macos]"');
    expect(validator).not.toContain('"manifest-path: src-tauri/Cargo.toml"');
    expect(validator).toContain(
      '"cargo deny --manifest-path src-tauri/Cargo.toml check advisories licenses sources"',
    );
  });

  it("routes only release tags to the hardened Mac runner", () => {
    const workflow = read(".github/workflows/release.yml");
    expect(workflow).toContain(
      "runs-on: [self-hosted, macOS, ARM64, spool-ledger-release]",
    );
    expect(workflow).toContain("timeout-minutes: 90");
    expect(workflow).toContain('test "$GITHUB_REF_PROTECTED" = "true"');
    expect(workflow).toContain('test "$GITHUB_REF" = "refs/tags/$RELEASE_TAG"');
    expect(workflow).toContain(
      'git rev-parse "refs/tags/$RELEASE_TAG^{commit}"',
    );
    expect(workflow).toContain("commit_sha=$tag_sha");
    expect(
      workflow.match(/ref: \$\{\{ needs\.preflight\.outputs\.commit_sha \}\}/g),
    ).toHaveLength(4);
    expect(workflow).toContain("--target universal-apple-darwin");
    expect(workflow).toContain("lipo -archs");
    expect(workflow).toContain("LSMinimumSystemVersion");
    expect(workflow).toContain("CFBundleIconFile");
    expect(workflow).toContain('rm -rf "$GITHUB_WORKSPACE/source"');
    expect(workflow).not.toContain("Select-Object -Single");
    expect(workflow).toContain("$nsis.Count -ne 1");
    expect(workflow).toContain("$msi.Count -ne 1");
    expect(workflow).not.toMatch(/ubuntu[^\n]*build-macos/i);
    expect(workflow).not.toMatch(/linux|appimage/i);
  });

  it("creates draft releases and never publishes automatically", () => {
    const workflow = read(".github/workflows/release.yml");
    expect(workflow).toContain("gh release create");
    expect(workflow).toContain("--draft");
    expect(workflow).toContain("--verify-tag");
    expect(workflow).not.toMatch(
      /--draft=false|gh release edit[^\n]*--draft=false/,
    );
    expect(workflow).toMatch(/actions\/attest@[0-9a-f]{40} # v4/);
    expect(workflow).toContain("npm run visual:check");
    expect(workflow).not.toContain("cargo-deny-action@");
    expect(
      workflow.match(/cargo install cargo-deny --version 0\.20\.2 --locked/g),
    ).toHaveLength(2);
    expect(
      workflow.match(
        /cargo deny --manifest-path src-tauri\/Cargo\.toml check advisories licenses sources/g,
      ),
    ).toHaveLength(2);
    expect(workflow).toContain("release:contract -- --tag");
    expect(workflow).toContain("--notes-file generated-release-notes.md");
    expect(workflow).toContain("gh release delete-asset");
    expect(workflow).toContain("require_draft");
    expect(workflow).toContain(
      'git fetch --force origin "refs/tags/$RELEASE_TAG:refs/tags/$RELEASE_TAG"',
    );
    expect(workflow).toMatch(/attest-release:[\s\S]*?attestations: write/);
    expect(workflow).toMatch(/assemble-draft-release:[\s\S]*?contents: write/);
    expect(workflow).not.toMatch(
      /assemble-draft-release:[\s\S]*?attestations: write/,
    );
    expect(workflow).toContain("SHA256SUMS.txt");
    expect(workflow).toContain("cancel-in-progress: false");
  });

  it("keeps pull requests off all self-hosted runners", () => {
    const ci = read(".github/workflows/ci.yml");
    expect(ci).toContain("pull_request:");
    expect(ci).not.toContain("self-hosted");
    expect(ci).toContain("timeout-minutes: 75");
    expect(ci).not.toContain("cargo-deny-action@");
    expect(ci).toContain("cargo install cargo-deny --version 0.20.2 --locked");
    expect(ci).toContain(
      "cargo deny --manifest-path src-tauri/Cargo.toml check advisories licenses sources",
    );
  });

  it("runs macOS acceptance only from protected main without release mutation", () => {
    const workflow = read(".github/workflows/macos-acceptance.yml");
    expect(workflow).toContain("workflow_dispatch:");
    expect(workflow).not.toContain("pull_request:");
    expect(workflow).not.toContain("push:");
    expect(workflow).toContain('test "$GITHUB_REF" = "refs/heads/main"');
    expect(workflow).toContain('test "$GITHUB_REF_PROTECTED" = "true"');
    expect(workflow).toContain(
      "runs-on: [self-hosted, macOS, ARM64, spool-ledger-release]",
    );
    expect(workflow).toContain("ref: ${{ github.sha }}");
    expect(workflow).toContain("--target universal-apple-darwin");
    expect(workflow).toContain("lipo -archs");
    expect(workflow).toContain("LSMinimumSystemVersion");
    expect(workflow).toContain("CFBundleIconFile");
    expect(workflow).toContain("macos-SHA256SUMS.txt");
    expect(workflow).toContain('rm -rf "$GITHUB_WORKSPACE/source"');
    expect(workflow).not.toMatch(/gh release|actions\/attest|contents: write/);
  });

  it("declares the universal macOS bundle contract", () => {
    const config = JSON.parse(read("src-tauri/tauri.conf.json"));
    expect(config.bundle.icon).toContain("icons/icon.icns");
    expect(config.bundle.macOS.minimumSystemVersion).toBe("12.0");
  });
});
