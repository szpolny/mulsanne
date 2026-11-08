<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";

  /** The game's render of a livery. Renders nothing when the game has no image for it. */
  let {
    file,
    view,
    alt = "",
    class: className = "",
  }: { file: string | null | undefined; view: "front" | "thumb" | "top"; alt?: string; class?: string } = $props();

  const src = $derived(file ? convertFileSrc(`${view}.${file}`, "car-render") : null);
  let failed = $state<string | null>(null);
</script>

{#if src && src !== failed}
  <img {src} {alt} class={className} draggable="false" loading="lazy" onerror={() => (failed = src)} />
{/if}
