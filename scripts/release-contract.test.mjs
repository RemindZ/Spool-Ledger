import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  artifactNames,
  loadReleaseContract,
  parseReleaseTag,
  validateReleaseContract,
  validateReleaseMatrix,
} from "./release-contract.mjs";

const versions = {
  packageJson: "0.1.0",
  cargoToml: "0.1.0",
  tauriConfig: "0.1.0",
};

const matrix = {
  version: 1,
  platforms: [
    {
      id: "windows-x64",
      platform: "windows",
      architecture: "x64",
      release_enabled: true,
      unsigned: true,
      evidence_document: "docs/compatibility/windows-x64-baseline.md",
      artifact_kinds: ["nsis", "msi", "portable"],
    },
    {
      id: "macos-universal",
      platform: "macos",
      architecture: "universal",
      release_enabled: true,
      unsigned: true,
      evidence_document: "docs/compatibility/macos-universal-baseline.md",
      artifact_kinds: ["dmg"],
    },
  ],
};

const evidenceExists = () => true;

describe("release contract", () => {
  it("accepts only exact semantic version tags", () => {
    expect(parseReleaseTag("v0.1.0")).toBe("0.1.0");
    for (const tag of [
      "0.1.0",
      "v0.1",
      "v01.1.0",
      "v0.1.0-beta.1",
      "v0.1.0 ",
    ]) {
      expect(() => parseReleaseTag(tag)).toThrow(/release tag/i);
    }
  });

  it("rejects a tag that differs from any version authority", () => {
    expect(() =>
      validateReleaseContract({
        tag: "v0.1.0",
        versions: { ...versions, cargoToml: "0.1.1" },
        matrix,
        evidenceExists,
      }),
    ).toThrow(/Cargo\.toml.*0\.1\.1.*0\.1\.0/i);
  });

  it("validates pending matrix structure without enabling release", () => {
    const pending = structuredClone(matrix);
    pending.platforms.forEach((row) => (row.release_enabled = false));

    expect(
      validateReleaseMatrix(pending, evidenceExists, { requireEnabled: false }),
    ).toBeUndefined();
  });

  it("rejects duplicate, mistyped, and misidentified platform rows", () => {
    const duplicate = structuredClone(matrix);
    duplicate.platforms.push(structuredClone(duplicate.platforms[0]));
    expect(() => validateReleaseMatrix(duplicate, evidenceExists)).toThrow(
      /duplicate.*windows-x64/i,
    );

    const stringBoolean = structuredClone(matrix);
    stringBoolean.platforms[0].release_enabled = "false";
    expect(() => validateReleaseMatrix(stringBoolean, evidenceExists)).toThrow(
      /release_enabled.*boolean/i,
    );

    const wrongIdentity = structuredClone(matrix);
    wrongIdentity.platforms[1].architecture = "arm64";
    expect(() => validateReleaseMatrix(wrongIdentity, evidenceExists)).toThrow(
      /macos-universal.*architecture.*universal/i,
    );
  });

  it("binds enabled rows to accepted evidence without pending work", () => {
    const wrongEvidence = structuredClone(matrix);
    wrongEvidence.platforms[0].evidence_document = "package.json";
    expect(() => validateReleaseMatrix(wrongEvidence, evidenceExists)).toThrow(
      /windows-x64.*evidence.*windows-x64-baseline/i,
    );

    const pending = structuredClone(matrix);
    pending.platforms[1].pending = "Operator AMS acceptance";
    expect(() => validateReleaseMatrix(pending, evidenceExists)).toThrow(
      /macos-universal.*pending/i,
    );

    for (const value of ["", null]) {
      const emptyPending = structuredClone(matrix);
      emptyPending.platforms[1].pending = value;
      expect(() => validateReleaseMatrix(emptyPending, evidenceExists)).toThrow(
        /macos-universal.*pending/i,
      );
    }
  });

  it("requires accepted Windows and macOS evidence", () => {
    const disabledMac = structuredClone(matrix);
    disabledMac.platforms[1].release_enabled = false;
    expect(() =>
      validateReleaseContract({
        tag: "v0.1.0",
        versions,
        matrix: disabledMac,
        evidenceExists,
      }),
    ).toThrow(/macos-universal.*not release enabled/i);

    expect(() =>
      validateReleaseContract({
        tag: "v0.1.0",
        versions,
        matrix,
        evidenceExists: (path) => !path.includes("macos"),
      }),
    ).toThrow(/macos-universal.*evidence document/i);
  });

  it("rejects Linux and unexpected release platforms", () => {
    const withLinux = structuredClone(matrix);
    withLinux.platforms.push({
      id: "linux-x64",
      platform: "linux",
      architecture: "x64",
      release_enabled: true,
      unsigned: true,
      evidence_document: "docs/compatibility/linux.md",
      artifact_kinds: ["appimage"],
    });
    expect(() =>
      validateReleaseContract({
        tag: "v0.1.0",
        versions,
        matrix: withLinux,
        evidenceExists,
      }),
    ).toThrow(/unexpected release platform.*linux-x64/i);
  });

  it("produces deterministic unsigned artifact names", () => {
    expect(artifactNames("0.1.0")).toEqual([
      "Spool-Ledger-v0.1.0-windows-x64-setup.exe",
      "Spool-Ledger-v0.1.0-windows-x64.msi",
      "Spool-Ledger-v0.1.0-windows-x64-portable.exe",
      "Spool-Ledger-v0.1.0-macos-universal.dmg",
    ]);
  });

  it("loads all version authorities and evidence from disk", () => {
    const root = mkdtempSync(join(tmpdir(), "spool-ledger-release-"));
    mkdirSync(join(root, "src-tauri"), { recursive: true });
    mkdirSync(join(root, "docs", "compatibility"), { recursive: true });
    writeFileSync(
      join(root, "package.json"),
      JSON.stringify({ version: "0.1.0" }),
    );
    writeFileSync(
      join(root, "src-tauri", "Cargo.toml"),
      '[package]\nname = "spool-ledger"\nversion = "0.1.0"\n',
    );
    writeFileSync(
      join(root, "src-tauri", "tauri.conf.json"),
      JSON.stringify({ version: "0.1.0" }),
    );
    writeFileSync(
      join(root, "docs", "compatibility", "release-matrix.json"),
      JSON.stringify(matrix),
    );
    for (const row of matrix.platforms) {
      writeFileSync(join(root, row.evidence_document), "accepted\n");
    }

    expect(loadReleaseContract(root, "v0.1.0").version).toBe("0.1.0");
  });

  it("accepts the complete initial-release matrix", () => {
    expect(
      validateReleaseContract({
        tag: "v0.1.0",
        versions,
        matrix,
        evidenceExists,
      }),
    ).toEqual({
      version: "0.1.0",
      artifacts: artifactNames("0.1.0"),
      unsigned: true,
    });
  });
});
