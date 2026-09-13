import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const read = (path: string) => readFileSync(`${root}/${path}`, "utf8");

describe("Spool Ledger design v2 handoff", () => {
  it("uses the approved identity assets without redrawing the logo", () => {
    const app = read("src/App.svelte");
    const chrome = read("src/lib/components/AppChrome.svelte");

    expect(app).toContain(
      "<title>Spool Ledger · Bambu Filament Migrator</title>",
    );
    expect(chrome).toContain('src="/spool-ledger-lockup-transparent.png"');
    expect(chrome).toContain('alt="Spool Ledger · Bambu Filament Migrator"');
    expect(app).not.toContain('class="brand-spool"');
    expect(chrome).not.toContain('class="brand-spool"');
    expect(app).toContain('src="/spool-ledger-logo-light.png"');
    expect(app).toContain('src="/spool-ledger-logo-dark.png"');
  });

  it("preserves the approved production identity asset hashes", () => {
    const approved = {
      "spool-ledger-lockup-transparent.png":
        "5a725861c6979e0f1ab426e288bcb46e06c13e6b9e7bf0a96683de4e339d699d",
      "spool-ledger-logo-light.png":
        "0909f35d5bab20aa4bbcd24d6df05e689a11fe7d90917da4a00107df2010f4d0",
      "spool-ledger-logo-dark.png":
        "22faf6b1bfcfd573a764d383d1a7a0b20f5d12202bc5b7fa55d9f3d548ffc3e9",
      "spool-ledger-readme-hero.png":
        "cc482676f1aae5d32c59632a51d96d98e68005d037960e855b75c688fae7d235",
    };
    for (const [name, hash] of Object.entries(approved)) {
      const production = `${root}/public/${name}`;
      expect(existsSync(production), name).toBe(true);
      expect(
        createHash("sha256").update(readFileSync(production)).digest("hex"),
        name,
      ).toBe(hash);
    }
  });

  it("uses the approved green-black dark palette and stable navigation surface", () => {
    const css = read("src/app.css");

    expect(css.match(/--ground:\s*#0d1410/gi)).toHaveLength(2);
    expect(css.match(/--surface:\s*#131d18/gi)).toHaveLength(2);
    expect(css.match(/--surface-raised:\s*#1a2720/gi)).toHaveLength(2);
    expect(css.match(/--text:\s*#eef4f0/gi)).toHaveLength(2);
    expect(css.match(/--line:\s*#2d4136/gi)).toHaveLength(2);
    expect(css.match(/--bambu:\s*#10bd52/gi)).toHaveLength(2);
    expect(css).toMatch(
      /\.workspace-tabs\s*\{[^}]*background:\s*var\(--surface\)/s,
    );
  });

  it("styles source selection as a complete custom checkbox state family", () => {
    const sourceTable = read("src/lib/components/SourceTable.svelte");
    const css = read("src/app.css");

    expect(sourceTable).toContain('class="source-profile-check"');
    expect(css).toMatch(/\.source-profile-check\s*\{[^}]*appearance:\s*none/s);
    expect(css).toContain(".source-profile-check:hover");
    expect(css).toContain(".source-profile-check:checked");
    expect(css).toContain(".source-profile-check:focus-visible");
    expect(css).toContain(".source-profile-check:disabled");
  });

  it("uses square workflow tiles, registration notches, one help glyph, and one transition treatment", () => {
    const naming = read("src/lib/components/NamingPanel.svelte");
    const css = read("src/app.css");

    expect(naming).not.toContain("CircleHelp");
    expect(naming).toContain(
      '<span class="help-glyph" aria-hidden="true">?</span>',
    );
    expect(css).toMatch(
      /\.workflow-steps li span\s*\{[^}]*border-radius:\s*5px/s,
    );
    expect(css).toContain(".source-rail::before");
    expect(css).toContain(".destination-rail::before");
    expect(css).toMatch(
      /\.workspace-view\s*\{[^}]*animation:\s*workspace-view-in/s,
    );
    expect(css).toContain("@keyframes workspace-view-in");
  });

  it("keeps local design directories ignored", () => {
    const ignored = read(".gitignore").split(/\r?\n/);
    expect(ignored).toContain("/design-v2/");
    expect(ignored).toContain("/docs/design/");
  });

  it("keeps synthetic inventory and onboarding captures on the production identity", () => {
    const fixture = read("tests/visual/FixtureApp.svelte");

    expect(fixture).toContain('state === "initial" || state === "setup"');
    expect(fixture).toContain('src="/spool-ledger-lockup-transparent.png"');
    expect(fixture).toContain('src="/spool-ledger-logo-light.png"');
    expect(fixture).toContain('src="/spool-ledger-logo-dark.png"');
  });

  it("keeps the production design contract aligned with v2 identity and official-only safety", () => {
    const design = read("DESIGN.md");

    expect(design).toContain("# Spool Ledger Design Handoff");
    expect(design).toContain("Bambu Filament Migrator");
    expect(design).toContain("`#0D1410`");
    expect(design).toContain("first-run setup");
    expect(design).toContain("Target profile required");
    expect(design).toContain("Registered source row");
    expect(design).not.toContain("Show custom printers");
  });

  it("keeps the authoritative scope aligned with the shipped identity and setup contracts", () => {
    const scope = read("SCOPE.md");

    expect(scope).toContain("# Spool Ledger - Bambu Filament Migrator Scope");
    expect(scope).toContain("**Spool Ledger**");
    expect(scope).toContain("first-run setup");
    expect(scope).toContain("enabled official printers");
    expect(scope).toContain("installed printer artwork");
    expect(scope).toContain("Target profile required");
    expect(scope).toContain("workspace preferences schema version 2");
    expect(scope).toContain("Public product name: **Spool Ledger**");
    expect(scope).toContain(
      "Permanent descriptor: **Bambu Filament Migrator**",
    );
    expect(scope).not.toContain(
      "Public project name: **Bambu Filament Migrator**",
    );
  });

  it("keeps project instructions, documentation media, and demo data truthful", () => {
    const claude = read("CLAUDE.md");
    const readme = read("README.md");
    const fixture = read("tests/visual/FixtureApp.svelte");

    for (const phrase of [
      "Spool Ledger",
      "Bambu Filament Migrator",
      "Never write to live",
      "official printers",
      "created_local",
      "windows_subsystem",
      "--bundles nsis",
      "No em dashes",
    ]) {
      expect(claude, phrase).toContain(phrase);
    }
    for (const name of [
      "inventory-loading.png",
      "setup-workspace.png",
      "review-plan.png",
      "run-evidence.png",
      "restore-preview.png",
      "theme-comparison.png",
    ]) {
      expect(readme, name).toContain(`docs/screenshots/${name}`);
      expect(existsSync(`${root}/docs/screenshots/${name}`), name).toBe(true);
    }
    for (const name of [
      "spool-ledger-product-tour.mp4",
      "spool-ledger-product-tour-voiceover.mp4",
    ]) {
      expect(readme, name).not.toContain(`docs/demo/${name}`);
      expect(existsSync(`${root}/docs/demo/${name}`), name).toBe(true);
    }
    expect(existsSync(`${root}/docs/demo/MUSIC-LICENSE.md`)).toBe(true);
    expect(existsSync(`${root}/docs/demo/VOICEOVER-SCRIPT.md`)).toBe(true);
    expect(readme).not.toContain("Printer-selectable");
    expect(readme).not.toContain("AMS-ready");
    expect(fixture).not.toMatch(/Polymaker|Sunlu/i);
  });

  it("embeds the V4 showcase using its permanent GitHub attachment URL", () => {
    const readme = read("README.md").replaceAll("\r\n", "\n");
    expect(readme).toContain(
      "\n\nhttps://github.com/user-attachments/assets/b0ad00a4-eca3-4415-a858-2d15bdb434c0\n\n",
    );
    expect(readme).toContain("61-second showcase");
    expect(readme).toContain("temporary demonstration workspace");
    expect(readme).toContain(
      "Bambu Studio synchronization and your printer or AMS check come next",
    );
    expect(readme).not.toContain("private-user-images.githubusercontent.com");
    expect(readme).not.toContain("?jwt=");
  });

  it("keeps the README focused on one showcase and plain support guidance", () => {
    const readme = read("README.md");
    expect(readme).not.toContain("## See the workflow");
    expect(readme).not.toContain("## Product videos");
    expect(readme).not.toContain("docs/marketing/graphics/readme.gif");
    const support = readme
      .split("## Current support")[1]
      .split("## What it does")[0];
    expect(support).toContain("macOS needs more testing");
    expect(support).not.toMatch(
      /human acceptance|acceptance gate|owner authorized/i,
    );
  });

  it("uses an animated header with a reduced-motion still", () => {
    const readme = read("README.md");
    expect(readme).toContain("<picture>");
    expect(readme).toContain('media="(prefers-reduced-motion: reduce)"');
    expect(readme).toContain(
      'srcset="docs/marketing/graphics/spool-ledger-header.png"',
    );
    expect(readme).toContain(
      'src="docs/marketing/graphics/spool-ledger-header.gif"',
    );
    expect(readme).toContain(
      'type="image/webp" srcset="docs/marketing/graphics/spool-ledger-header.webp"',
    );
    const webp = readFileSync(
      `${root}/docs/marketing/graphics/spool-ledger-header.webp`,
    );
    expect(webp.subarray(0, 4).toString()).toBe("RIFF");
    expect(webp.subarray(8, 12).toString()).toBe("WEBP");
    expect(webp.includes(Buffer.from("ANIM"))).toBe(true);
    expect(webp.length).toBeLessThan(5_000_000);
    const animation = readFileSync(
      `${root}/docs/marketing/graphics/spool-ledger-header.gif`,
    );
    expect(animation.subarray(0, 6).toString()).toBe("GIF89a");
    expect(animation.readUInt16LE(6)).toBe(1440);
    expect(animation.readUInt16LE(8)).toBe(448);
    expect(animation.includes(Buffer.from("NETSCAPE2.0"))).toBe(true);
    expect(animation.length).toBeLessThan(10_000_000);
  });

  it("preserves the wide Blender header and its supplied-logo provenance", () => {
    const header = readFileSync(
      `${root}/docs/marketing/graphics/spool-ledger-header.png`,
    );
    expect(header.subarray(1, 4).toString()).toBe("PNG");
    expect(header.readUInt32BE(16)).toBe(1800);
    expect(header.readUInt32BE(20)).toBe(560);
    expect(header.length).toBeLessThan(1_000_000);
    expect(createHash("sha256").update(header).digest("hex")).toBe(
      "827012ebdbc52806bd480200892488f692af3789fe15366d7ad48088625dfe05",
    );
    const disclosure = read("docs/marketing/AI-DISCLOSURE.md");
    expect(disclosure).toContain("Blender 5.0.1");
    expect(disclosure).toContain("public/spool-ledger-lockup-transparent.png");
    expect(disclosure).toContain(
      "logo was not traced, redrawn or reconstructed",
    );
  });

  it("makes Spool Ledger dominant in repository and desktop identity surfaces", () => {
    const readme = read("README.md");
    const tauri = JSON.parse(read("src-tauri/tauri.conf.json"));

    expect(readme).toContain(
      'src="docs/marketing/graphics/spool-ledger-header.gif"',
    );
    expect(readme).not.toContain('src="public/spool-ledger-readme-hero.png"');
    expect(readme).toContain('alt="Spool Ledger · Bambu Filament Migrator,');
    expect(readme).not.toContain(
      "approved product design, interactive UI prototype",
    );
    expect(tauri.productName).toBe("Spool Ledger");
    expect(tauri.app.windows[0].title).toBe(
      "Spool Ledger · Bambu Filament Migrator",
    );
  });
});
