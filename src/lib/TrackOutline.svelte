<script lang="ts">
  import { trackOutline } from "./tracks";

  let { track, class: cls = "", hideMissing = false }: { track: string; class?: string; hideMissing?: boolean } = $props();

  const outline = $derived(trackOutline(track));
</script>

{#if outline}
  <svg class={cls} viewBox={outline.viewBox} preserveAspectRatio="xMidYMid meet" role="img" aria-label="{outline.name} layout">
    <path d={outline.d} fill="none" stroke="currentColor" stroke-width="4" stroke-linejoin="round" stroke-linecap="round" />
  </svg>
{:else if !hideMissing}
  <div class={["flex items-center justify-center text-[11px] text-faint", cls]}>No map</div>
{/if}
