<script lang="ts">
  import { compoundStyle } from "../lib/brands";

  let {
    compound,
    wear,
    laps,
    show = "wear",
  }: { compound: string; wear: number | null; laps: number; show?: "wear" | "laps" } = $props();

  const style = $derived(compoundStyle(compound));
  /** Tread used, in percent. */
  const used = $derived(wear == null ? null : Math.round((1 - wear) * 100));
</script>

<span class="inline-flex items-center gap-1.5" title="{compound || 'Unknown'} · {used ?? '?'}% used · {laps} laps">
  {#if compound}
    <span
      class="inline-flex h-[18px] w-[18px] items-center justify-center rounded-full text-[10px] leading-none font-bold"
      style:border="2.5px solid {style.color}"
      style:color={style.color}>{style.letter}</span
    >
  {/if}
  <span class="w-7 text-right text-[12px] opacity-80">
    {#if show === "laps" || used == null}{laps}L{:else}{used}%{/if}
  </span>
</span>
