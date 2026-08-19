<script lang="ts">
  const version = '0.1.0';
</script>

<svelte:head>
  <meta name="description" content="Migrate filament presets safely" />
</svelte:head>

<main class="shell">
  <header class="topbar">
    <div>
      <p class="eyebrow">LOCAL PROFILE TOOL</p>
      <h1>Bambu Filament Migrator</h1>
    </div>
    <span class="version">v{version}</span>
  </header>
  <section class="empty-state" aria-labelledby="setup-title">
    <div class="spool" aria-hidden="true"></div>
    <h2 id="setup-title">Profile engine is starting up</h2>
    <p>Discovery and migration controls will appear here.</p>
  </section>
</main>
