<script lang="ts">
  import type { Snippet } from "svelte";

  /** Title row shared by every companion page: optional eyebrow, title, description and actions on the right. */
  let {
    title,
    eyebrow,
    description,
    actions,
    children,
  }: { title: string; eyebrow?: string; description?: string; actions?: Snippet; children?: Snippet } = $props();
</script>

<header class="mb-6 flex flex-wrap items-end justify-between gap-x-6 gap-y-3">
  <div class="min-w-0">
    {#if eyebrow}<p class="mb-1.5 eyebrow">{eyebrow}</p>{/if}
    <h1 class="page-title">{title}</h1>
    {#if description}<p class="mt-1.5 max-w-[46em] text-sm text-soft">{description}</p>{/if}
    {@render children?.()}
  </div>
  {#if actions}
    <div class="flex shrink-0 flex-wrap items-center gap-2">{@render actions()}</div>
  {/if}
</header>
