import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  loadReleaseContract,
  validateReleaseMatrix,
} from "./release-contract.mjs";

const root = resolve(import.meta.dirname, "..");

function read(relativePath) {
  return readFileSync(resolve(root, relativePath), "utf8");
}

function requireText(relativePath, fragments) {
  const content = read(relativePath);
  for (const fragment of fragments) {
    if (!content.includes(fragment)) {
      throw new Error(`${relativePath} is missing required text: ${fragment}`);
    }
  }
  return content;
}

const workflow = requireText(".github/workflows/ci.yml", [
  "runs-on: windows-latest",
  "npm ci",
  "npm run format:check",
  "npm run lint",
  "npm test -- --run",
  "npm run check",
  "npm run tauri build -- --no-bundle",
  "cargo fmt --manifest-path src-tauri/Cargo.toml -- --check",
  "cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings",
  "cargo test --manifest-path src-tauri/Cargo.toml --all-targets",
  "Get-FileHash",
  "actions/upload-artifact@",
  "cargo deny --manifest-path src-tauri/Cargo.toml check advisories licenses sources",
]);
if (/runs-on:\s*(?:macos|ubuntu)/.test(workflow)) {
  throw new Error("CI must not route pull requests to macOS or Linux runners");
}
if (workflow.includes("self-hosted")) {
  throw new Error("CI must not route pull requests to self-hosted runners");
}

const releaseWorkflow = requireText(".github/workflows/release.yml", [
  'tags: ["v*"]',
  "preflight:",
  "build-windows:",
  "build-macos:",
  "assemble-release:",
  "attest-release:",
  "assemble-draft-release:",
  "runs-on: [self-hosted, macOS, ARM64, spool-ledger-release]",
  "needs: [preflight, build-windows, build-macos]",
  "--target universal-apple-darwin",
  "--draft",
  "--verify-tag",
  "actions/attest@",
  "SHA256SUMS.txt",
]);
if (releaseWorkflow.includes("pull_request:")) {
  throw new Error("release workflow must never run for pull requests");
}
if (/linux|appimage/i.test(releaseWorkflow)) {
  throw new Error("Linux must remain outside the initial release workflow");
}

validateReleaseMatrix(
  JSON.parse(read("docs/compatibility/release-matrix.json")),
  (path) => existsSync(resolve(root, path)),
  { requireEnabled: false },
);

requireText("README.md", [
  "Windows 11 x64",
  "Not release-supported",
  "requests no Bambu credentials",
  "PFUS",
  "Restore",
  "AGPL-3.0",
]);
requireText("SECURITY.md", [
  "Reporting a vulnerability",
  "live slicer profile",
]);
requireText("CONTRIBUTING.md", [
  "synthetic fixtures",
  "cargo fmt",
  "npm run format:check",
]);
requireText("NOTICE.md", [
  "Copyright (C) 2026 Matthias (RemindZ)",
  "not affiliated with or endorsed by Bambu Lab",
  "Fixture provenance",
]);
requireText("deny.toml", [
  "[advisories]",
  "[licenses]",
  "[sources]",
  '"AGPL-3.0-only"',
  '"x86_64-pc-windows-msvc"',
  '"aarch64-apple-darwin"',
  '"x86_64-apple-darwin"',
]);
requireText("LICENSE", ["Copyright (C) 2026 Matthias (RemindZ)"]);

const packageJson = JSON.parse(read("package.json"));
const tauriConfig = JSON.parse(read("src-tauri/tauri.conf.json"));
if (packageJson.version !== tauriConfig.version) {
  throw new Error("package.json and tauri.conf.json versions differ");
}
const cargoPackage = read("src-tauri/Cargo.toml").match(
  /\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1];
if (cargoPackage !== packageJson.version) {
  throw new Error("package.json and Cargo.toml versions differ");
}
if (packageJson.dependencies?.["@tauri-apps/plugin-opener"] === undefined) {
  throw new Error("Tauri opener JavaScript dependency is missing");
}
if (!read("src-tauri/Cargo.toml").includes("tauri-plugin-opener")) {
  throw new Error("Tauri opener Rust dependency is missing");
}
if (!tauriConfig.bundle?.icon?.includes("icons/icon.icns")) {
  throw new Error("macOS ICNS bundle icon is missing");
}
if (tauriConfig.bundle?.macOS?.minimumSystemVersion !== "12.0") {
  throw new Error("macOS minimum system version must be explicit");
}
if (tauriConfig.bundle?.copyright !== "Copyright (C) 2026 Matthias (RemindZ)") {
  throw new Error("Tauri bundle copyright is missing or incorrect");
}
if (tauriConfig.bundle?.licenseFile !== "../LICENSE") {
  throw new Error("Tauri bundle must include the AGPL license file");
}

requireText("src-tauri/src/main.rs", [
  '#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]',
]);
const csp = tauriConfig.app?.security?.csp ?? "";
const imageSources = csp
  .split(";")
  .map((directive) => directive.trim())
  .find((directive) => directive.startsWith("img-src "));
if (!imageSources?.split(/\s+/).includes("blob:")) {
  throw new Error("Tauri img-src CSP must allow generated blob artwork URLs");
}

requireText("src-tauri/capabilities/default.json", [
  "opener:allow-open-url",
  "https://buymeacoffee.com/Remitec",
]);
requireText("docs/release/RELEASING.md", [
  "draft",
  "SHA256SUMS.txt",
  "gh attestation verify",
  "playable README video",
]);
requireText("docs/release/self-hosted-macos-runner.md", [
  "spool-ledger-release",
  "Pull requests",
  "dedicated non-admin",
]);
requireText("docs/release/unsigned-release-notice.md", [
  "Unsigned initial release",
  "SmartScreen",
  "Gatekeeper",
]);

const tagIndex = process.argv.indexOf("--tag");
if (tagIndex >= 0) {
  const tag = process.argv[tagIndex + 1];
  if (!tag) throw new Error("--tag requires vX.Y.Z");
  loadReleaseContract(root, tag);
}

console.log("release policy validation passed");
