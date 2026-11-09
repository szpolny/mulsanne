<script lang="ts">
  import { compoundStyle } from "../lib/brands";

  /** `compounds` is ordered FL, FR, RL, RR. */
  let { compounds, wear, laps }: { compounds: string[]; wear: number | null; laps: number } = $props();

  const CORNERS = ["FL", "FR", "RL", "RR"];
  const mixed = $derived(new Set(compounds).size > 1);
  const name = $derived(mixed ? compounds.map((c, i) => `${CORNERS[i]} ${c || "?"}`).join(", ") : compounds[0] || "Unknown");
  /** Tread used, in percent. */
  const used = $derived(wear == null ? null : Math.round((1 - wear) * 100));
</script>

{#snippet badge(compound: string, small = false)}
  {@const style = compoundStyle(compound)}
  <span
    class={[
      "inline-flex items-center justify-center rounded-full bg-black/85 leading-none font-bold",
      small ? "h-2 w-2 text-[6px]" : "h-[18px] w-[18px] text-[10px]",
    ]}
    style:border="{small ? 1 : 2.5}px solid {style.color}"
    style:color={style.color}>{style.letter}</span
  >
{/snippet}

<!-- Always one badge wide, so the column keeps its width and every car's badges share a centre line. -->
<span class="flex h-[18px] w-[18px] items-center justify-center" title="{name} · {used ?? '?'}% used · {laps} laps">
  {#if mixed}
    <!-- One badge per corner, laid out like the car seen from above: 8 + 2 + 8 fills the single badge's 18px. -->
    <span class="grid grid-cols-2 gap-[2px]">
      {#each compounds as compound}{@render badge(compound, true)}{/each}
    </span>
  {:else if compounds[0]}
    {@render badge(compounds[0])}
  {/if}
</span>
