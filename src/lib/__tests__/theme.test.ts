// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import ThemeToggle from "../components/ThemeToggle.svelte";
import { nextTheme, themeCycle, themeLabel } from "../theme";

describe("theme cycle", () => {
  it("goes to the opposite of the resolved system theme first", () => {
    expect(themeCycle("light")).toEqual(["system", "dark", "light"]);
    expect(themeCycle("dark")).toEqual(["system", "light", "dark"]);
    expect(nextTheme("system", "light")).toBe("dark");
    expect(nextTheme("dark", "light")).toBe("light");
    expect(nextTheme("light", "light")).toBe("system");
    expect(nextTheme("system", "dark")).toBe("light");
    expect(nextTheme("light", "dark")).toBe("dark");
    expect(nextTheme("dark", "dark")).toBe("system");
  });

  it("names the current mode and the next one", () => {
    expect(themeLabel("system", "dark")).toEqual({
      title: "Theme: Automatic (dark)",
      label: "Theme: Automatic (dark). Switch to Light",
    });
    expect(themeLabel("light", "light")).toEqual({
      title: "Theme: Light",
      label: "Theme: Light. Switch to Automatic",
    });
    expect(themeLabel("dark", "light").label).toBe(
      "Theme: Dark. Switch to Light",
    );
  });
});

describe("ThemeToggle", () => {
  let listeners: Array<(event: { matches: boolean }) => void> = [];
  let systemDark = false;

  function mockSystem(dark: boolean) {
    systemDark = dark;
    listeners = [];
    vi.stubGlobal("matchMedia", (query: string) => ({
      get matches() {
        return query.includes("dark") ? systemDark : false;
      },
      media: query,
      addEventListener: (
        _type: string,
        listener: (event: { matches: boolean }) => void,
      ) => listeners.push(listener),
      removeEventListener: (
        _type: string,
        listener: (event: { matches: boolean }) => void,
      ) => {
        listeners = listeners.filter((item) => item !== listener);
      },
    }));
  }

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });

  it("cycles Automatic, Dark, Light on a light system", async () => {
    mockSystem(false);
    const onChange = vi.fn();
    const view = render(ThemeToggle, { theme: "system", onChange });
    const button = screen.getByRole("button", {
      name: "Theme: Automatic (light). Switch to Dark",
    });
    expect(button.dataset.mode).toBe("system");
    await fireEvent.click(button);
    expect(onChange).toHaveBeenLastCalledWith("dark");
    await view.rerender({ theme: "dark", onChange });
    await fireEvent.click(button);
    expect(onChange).toHaveBeenLastCalledWith("light");
    await view.rerender({ theme: "light", onChange });
    await fireEvent.click(button);
    expect(onChange).toHaveBeenLastCalledWith("system");
  });

  it("cycles Automatic, Light, Dark on a dark system", async () => {
    mockSystem(true);
    const onChange = vi.fn();
    render(ThemeToggle, { theme: "system", onChange });
    await fireEvent.click(
      screen.getByRole("button", {
        name: "Theme: Automatic (dark). Switch to Light",
      }),
    );
    expect(onChange).toHaveBeenLastCalledWith("light");
  });

  it("follows an OS theme change while on Automatic", async () => {
    mockSystem(false);
    const onChange = vi.fn();
    render(ThemeToggle, { theme: "system", onChange });
    systemDark = true;
    for (const listener of listeners) listener({ matches: true });
    const button = await screen.findByRole("button", {
      name: "Theme: Automatic (dark). Switch to Light",
    });
    expect(button.title).toBe("Theme: Automatic (dark)");
    await fireEvent.click(button);
    expect(onChange).toHaveBeenLastCalledWith("light");
  });
});
