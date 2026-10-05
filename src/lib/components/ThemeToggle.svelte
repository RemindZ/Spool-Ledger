<script lang="ts">
  import {
    SYSTEM_DARK_QUERY,
    nextTheme,
    systemTheme,
    themeLabel,
    type ResolvedTheme,
    type Theme,
  } from "../theme";

  let {
    theme,
    onChange,
  }: {
    theme: Theme;
    onChange: (theme: Theme) => void;
  } = $props();

  let system = $state<ResolvedTheme>(systemTheme());
  const label = $derived(themeLabel(theme, system));

  $effect(() => {
    if (typeof matchMedia !== "function") return;
    const query = matchMedia(SYSTEM_DARK_QUERY);
    const update = () => (system = systemTheme());
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  });
</script>

<button
  type="button"
  class="theme-toggle"
  data-mode={theme}
  title={label.title}
  aria-label={label.label}
  onclick={() => onChange(nextTheme(theme, system))}
>
  <svg
    class="theme-icon theme-icon-system"
    viewBox="0 0 24 24"
    aria-hidden="true"
    ><circle cx="12" cy="12" r="8.5" /><path
      d="M12 3.5v17a8.5 8.5 0 0 0 0-17Z"
      fill="currentColor"
      stroke="none"
    /></svg
  >
  <svg
    class="theme-icon theme-icon-light"
    viewBox="0 0 24 24"
    aria-hidden="true"
    ><circle cx="12" cy="12" r="4" /><path
      d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M5.3 18.7l1.6-1.6M17.1 6.9l1.6-1.6"
    /></svg
  >
  <svg class="theme-icon theme-icon-dark" viewBox="0 0 24 24" aria-hidden="true"
    ><path d="M19.5 14.6A8 8 0 0 1 9.4 4.5a8 8 0 1 0 10.1 10.1Z" /></svg
  >
</button>
