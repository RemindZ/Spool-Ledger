export type Theme = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const NAMES: Record<Theme, string> = {
  system: "Automatic",
  light: "Light",
  dark: "Dark",
};

/** Automatic, then the opposite of what Automatic resolves to, then the rest. */
export function themeCycle(system: ResolvedTheme): Theme[] {
  return ["system", system === "dark" ? "light" : "dark", system];
}

export function nextTheme(current: Theme, system: ResolvedTheme): Theme {
  const cycle = themeCycle(system);
  return cycle[(cycle.indexOf(current) + 1) % cycle.length];
}

export function themeLabel(
  theme: Theme,
  system: ResolvedTheme,
): { title: string; label: string } {
  const current = theme === "system" ? `Automatic (${system})` : NAMES[theme];
  return {
    title: `Theme: ${current}`,
    label: `Theme: ${current}. Switch to ${NAMES[nextTheme(theme, system)]}`,
  };
}

export const SYSTEM_DARK_QUERY = "(prefers-color-scheme: dark)";

export function systemTheme(): ResolvedTheme {
  return typeof matchMedia === "function" &&
    matchMedia(SYSTEM_DARK_QUERY).matches
    ? "dark"
    : "light";
}
