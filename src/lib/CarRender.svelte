<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { Snippet } from "svelte";

  /**
   * The game's render of a livery. Given several, shows the first the game has an
   * image for. When it has none, shows `fallback`, or nothing.
   */
  let {
    file,
    view,
    alt = "",
    class: className = "",
    fallback,
  }: {
    file: string | readonly string[] | null | undefined;
    view: "front" | "thumb" | "top";
    alt?: string;
    class?: string;
    fallback?: Snippet;
  } = $props();

  const srcs = $derived((typeof file === "string" ? [file] : (file ?? [])).filter(Boolean).map((f) => convertFileSrc(`${view}.${f}`, "car-render")));
  let failed = $state<string[]>([]);
  const src = $derived(srcs.find((s) => !failed.includes(s)) ?? null);
</script>

{#if src}
  <img {src} {alt} class={className} draggable="false" loading="lazy" onerror={() => (failed = [...failed, src])} />
{:else if fallback}
  {@render fallback()}
{/if}
