<script lang="ts">
  import { compoundStyle } from "../lib/brands";

  let { compound, wear, laps }: { compound: string; wear: number | null; laps: number } = $props();

  const style = $derived(compoundStyle(compound));
  /** Tread used, in percent. */
  const used = $derived(wear == null ? null : Math.round((1 - wear) * 100));
</script>

<span class="inline-flex items-center" title="{compound || 'Unknown'} · {used ?? '?'}% used · {laps} laps">
  {#if compound}
    <span
      class="inline-flex h-[18px] w-[18px] items-center justify-center rounded-full text-[10px] leading-none font-bold"
      style:border="2.5px solid {style.color}"
      style:color={style.color}>{style.letter}</span
    >
  {/if}
</span>
