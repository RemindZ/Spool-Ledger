<script lang="ts">
  import { Printer } from "@lucide/svelte";
  import { api } from "../api";
  import type { PrinterTarget } from "../types";

  let {
    catalogId,
    printer,
  }: {
    catalogId: string;
    printer: PrinterTarget;
  } = $props();

  let artworkUrl = $state<string | null>(null);

  $effect(() => {
    const available = printer.artwork_available && printer.kind === "official";
    const requestedCatalogId = catalogId;
    const requestedPrinterId = printer.id;
    let active = true;
    let createdUrl: string | null = null;
    artworkUrl = null;

    if (available) {
      void api
        .printerArtwork(requestedCatalogId, requestedPrinterId)
        .then((bytes) => {
          if (!active) return;
          createdUrl = URL.createObjectURL(
            new Blob([bytes], { type: "image/png" }),
          );
          artworkUrl = createdUrl;
        })
        .catch(() => {
          if (active) artworkUrl = null;
        });
    }

    return () => {
      active = false;
      if (createdUrl) URL.revokeObjectURL(createdUrl);
    };
  });
</script>

<span class="printer-artwork">
  {#if artworkUrl}
    <img src={artworkUrl} alt={printer.name} />
  {:else}
    <span data-testid="printer-artwork-fallback" aria-hidden="true">
      <Printer size={28} strokeWidth={1.45} />
    </span>
  {/if}
</span>

<style>
  .printer-artwork,
  .printer-artwork > span {
    display: grid;
    width: 3.25rem;
    height: 3.25rem;
    flex: 0 0 auto;
    place-items: center;
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .printer-artwork > span {
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    background: var(--surface-raised);
  }
</style>
