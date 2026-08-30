import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REQUIRED_PLATFORMS = new Map([
  [
    "windows-x64",
    {
      platform: "windows",
      architecture: "x64",
      evidenceDocument: "docs/compatibility/windows-x64-baseline.md",
      artifactKinds: ["nsis", "msi", "portable"],
    },
  ],
  [
    "macos-universal",
    {
      platform: "macos",
      architecture: "universal",
      evidenceDocument: "docs/compatibility/macos-universal-baseline.md",
      artifactKinds: ["dmg"],
    },
  ],
]);

export function parseReleaseTag(tag) {
  const match = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.exec(tag);
  if (!match) {
    throw new Error(`invalid release tag: ${tag}`);
  }
  return match.slice(1).join(".");
}

export function artifactNames(version) {
  return [
    `Spool-Ledger-v${version}-windows-x64-setup.exe`,
    `Spool-Ledger-v${version}-windows-x64.msi`,
    `Spool-Ledger-v${version}-windows-x64-portable.exe`,
    `Spool-Ledger-v${version}-macos-universal.dmg`,
  ];
}

export function validateReleaseMatrix(
  matrix,
  evidenceExists,
  { requireEnabled = true } = {},
) {
  if (matrix?.version !== 1 || !Array.isArray(matrix.platforms)) {
    throw new Error("release matrix must use schema version 1");
  }
  const byId = new Map();
  for (const row of matrix.platforms) {
    if (!REQUIRED_PLATFORMS.has(row.id)) {
      throw new Error(`unexpected release platform: ${row.id}`);
    }
    if (byId.has(row.id)) {
      throw new Error(`duplicate release platform: ${row.id}`);
    }
    byId.set(row.id, row);
  }
  for (const [id, expected] of REQUIRED_PLATFORMS) {
    const row = byId.get(id);
    if (!row) {
      throw new Error(`${id} is missing from the release matrix`);
    }
    if (row.platform !== expected.platform) {
      throw new Error(`${id} platform must be ${expected.platform}`);
    }
    if (row.architecture !== expected.architecture) {
      throw new Error(`${id} architecture must be ${expected.architecture}`);
    }
    if (typeof row.release_enabled !== "boolean") {
      throw new Error(`${id} release_enabled must be a boolean`);
    }
    if (typeof row.unsigned !== "boolean") {
      throw new Error(`${id} unsigned must be a boolean`);
    }
    if (row.evidence_document !== expected.evidenceDocument) {
      throw new Error(
        `${id} evidence document must be ${expected.evidenceDocument}`,
      );
    }
    const hasPending = Object.hasOwn(row, "pending");
    if (hasPending && typeof row.pending !== "string") {
      throw new Error(`${id} pending must be a string when present`);
    }
    if (row.release_enabled && hasPending) {
      throw new Error(`${id} cannot be release enabled while work is pending`);
    }
    if (requireEnabled && !row.release_enabled) {
      throw new Error(`${id} is not release enabled`);
    }
    if (!row.unsigned) {
      throw new Error(
        `${id} must remain marked unsigned until signing is configured`,
      );
    }
    if (!row.evidence_document || !evidenceExists(row.evidence_document)) {
      throw new Error(
        `${id} evidence document is missing: ${row.evidence_document ?? "none"}`,
      );
    }
    if (
      JSON.stringify(row.artifact_kinds) !==
      JSON.stringify(expected.artifactKinds)
    ) {
      throw new Error(
        `${id} artifact kinds do not match the initial release contract`,
      );
    }
  }
}

export function validateReleaseContract({
  tag,
  versions,
  matrix,
  evidenceExists,
}) {
  const version = parseReleaseTag(tag);
  const authorities = [
    ["package.json", versions.packageJson],
    ["Cargo.toml", versions.cargoToml],
    ["tauri.conf.json", versions.tauriConfig],
  ];
  for (const [name, actual] of authorities) {
    if (actual !== version) {
      throw new Error(
        `${name} version ${actual} does not match release ${version}`,
      );
    }
  }

  validateReleaseMatrix(matrix, evidenceExists);

  return { version, artifacts: artifactNames(version), unsigned: true };
}

function cargoPackageVersion(content) {
  const start = content.indexOf("[package]");
  const remainder = start < 0 ? "" : content.slice(start + "[package]".length);
  const nextSection = remainder.search(/^\[/m);
  const packageSection =
    nextSection < 0 ? remainder : remainder.slice(0, nextSection);
  const version = /^version\s*=\s*"([^"]+)"\s*$/m.exec(packageSection)?.[1];
  if (!version)
    throw new Error("src-tauri/Cargo.toml package version is missing");
  return version;
}

export function loadReleaseContract(root, tag) {
  const read = (path) => readFileSync(resolve(root, path), "utf8");
  const packageJson = JSON.parse(read("package.json"));
  const tauriConfig = JSON.parse(read("src-tauri/tauri.conf.json"));
  const matrix = JSON.parse(read("docs/compatibility/release-matrix.json"));
  return validateReleaseContract({
    tag,
    versions: {
      packageJson: packageJson.version,
      cargoToml: cargoPackageVersion(read("src-tauri/Cargo.toml")),
      tauriConfig: tauriConfig.version,
    },
    matrix,
    evidenceExists: (path) => existsSync(resolve(root, path)),
  });
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : "";
if (invokedPath === fileURLToPath(import.meta.url)) {
  const tagIndex = process.argv.indexOf("--tag");
  if (tagIndex < 0 || !process.argv[tagIndex + 1]) {
    console.error("usage: node scripts/release-contract.mjs --tag vX.Y.Z");
    process.exit(2);
  }
  try {
    const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
    const result = loadReleaseContract(root, process.argv[tagIndex + 1]);
    console.log(JSON.stringify(result));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
