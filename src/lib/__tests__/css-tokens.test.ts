import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const componentNames = [
  "FirstRunSetup",
  "PlanDependencyDialog",
  "PrinterArtwork",
  "PrinterManagerDialog",
  "PrinterPicker",
  "TargetPanel",
];

describe("component theme tokens", () => {
  it("defines every CSS custom property used by the new printer workflow", () => {
    const theme = readFileSync(`${root}/src/app.css`, "utf8");
    const defined = new Set(
      [...theme.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((match) => match[1]),
    );
    const used = new Set(
      componentNames.flatMap((name) => {
        const source = readFileSync(
          `${root}/src/lib/components/${name}.svelte`,
          "utf8",
        );
        return [...source.matchAll(/var\((--[a-z0-9-]+)\)/g)].map(
          (match) => match[1],
        );
      }),
    );

    expect([...used].filter((token) => !defined.has(token))).toEqual([]);
  });
});
