<script lang="ts">
  import { compoundStyle } from "../lib/brands";

  /** `compounds` is ordered FL, FR, RL, RR. */
  let { compounds, wear, laps }: { compounds: string[]; wear: number | null; laps: number } = $props();

  const CORNERS = ["FL", "FR", "RL", "RR"];
  const mixed = $derived(new Set(compounds).size > 1);
  const style = $derived(compoundStyle(compounds[0] ?? ""));
  const name = $derived(mixed ? compounds.map((c, i) => `${CORNERS[i]} ${c || "?"}`).join(", ") : compounds[0] || "Unknown");
  /** Tread used, in percent. */
  const used = $derived(wear == null ? null : Math.round((1 - wear) * 100));
</script>

<span class="inline-flex items-center" title="{name} · {used ?? '?'}% used · {laps} laps">
  {#if mixed}
    <!-- One cell per corner, laid out like the car seen from above. The backing keeps red cells visible on the red player row. -->
    <span class="grid h-[18px] w-[18px] grid-cols-2 gap-px rounded-[3px] bg-black p-px">
      {#each compounds as compound}
        {@const corner = compoundStyle(compound)}
        <span
          class="flex items-center justify-center rounded-[2px] text-[8px] leading-none font-bold text-black"
          style:background={corner.color}>{corner.letter}</span
        >
      {/each}
    </span>
  {:else if compounds[0]}
    <span
      class="inline-flex h-[18px] w-[18px] items-center justify-center rounded-full bg-black/85 text-[10px] leading-none font-bold"
      style:border="2.5px solid {style.color}"
      style:color={style.color}>{style.letter}</span
    >
  {/if}
</span>
