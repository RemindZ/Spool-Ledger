import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const fixturePath = join(root, "tests", "visual", "FixtureApp.svelte");
const screenshots = join(root, "tests", "visual", "screenshots");
const fixture = readFileSync(fixturePath, "utf8");
const states = [
  "initial",
  "no-account",
  "ready",
  "naming",
  "plan",
  "conflict",
  "execution",
  "monitoring",
  "timeout",
  "cloud",
  "ams",
  "restore",
  "narrow",
  "setup",
  "manager",
  "dependency",
  "artwork-fallback",
];

if (!fixture.includes("onOpenSupport={async () => {}}")) {
  throw new Error(
    "completed-run visual fixtures must include the support action",
  );
}

for (const [index, state] of states.entries()) {
  if (!fixture.includes(`["${state}",`)) {
    throw new Error(`visual fixture is missing state: ${state}`);
  }
  const number = String(index + 1).padStart(2, "0");
  for (const theme of ["light", "dark"]) {
    const file = join(screenshots, `visual-${number}-${state}-${theme}.png`);
    if (!existsSync(file) || statSync(file).size === 0) {
      throw new Error(`visual screenshot is missing or empty: ${file}`);
    }
  }
}

const indexHtml = readFileSync(join(root, "index.html"), "utf8");
if (indexHtml.includes("tests/visual")) {
  throw new Error("production index imports the visual fixture");
}

const dist = join(root, "dist");
if (existsSync(dist)) {
  const files = readdirSync(join(dist, "assets")).map((name) =>
    readFileSync(join(dist, "assets", name), "utf8"),
  );
  if (files.some((content) => content.includes("fixture-plan-2026-08-21"))) {
    throw new Error("visual fixture data leaked into the production bundle");
  }
}

console.log(
  `visual fixture: ${states.length} states, ${states.length * 2} theme captures, production entry isolated`,
);
