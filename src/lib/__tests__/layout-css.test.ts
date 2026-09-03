import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const css = readFileSync(`${root}/src/app.css`, "utf8");

describe("desktop layout containment", () => {
  it("allows long migration plan IDs to wrap inside modal dialogs", () => {
    expect(css).toMatch(
      /\.modal-dialog-surface\s*>\s*\*\s*\{[^}]*min-width:\s*0;/s,
    );
    expect(css).toMatch(
      /\.modal-dialog-surface\s+h2\s*\{[^}]*overflow-wrap:\s*anywhere;/s,
    );
  });

  it("keeps the setup workspace within the available desktop viewport", () => {
    expect(css).toMatch(
      /\.workspace-shell:has\(\.setup-intro\)\s*\{[^}]*height:\s*calc\(100dvh\s*-\s*142px\);[^}]*overflow:\s*hidden;/s,
    );
    expect(css).toMatch(
      /\.workspace-view:has\(>\s*\.setup-intro\)\s*\{[^}]*grid-template-rows:\s*auto\s+minmax\(0,\s*1fr\)\s+auto;/s,
    );
    expect(css).toMatch(
      /\.workspace-view:has\(>\s*\.setup-intro\)\s+\.(?:naming|destination)-rail[\s\S]*overflow-y:\s*auto;/,
    );
  });
});
