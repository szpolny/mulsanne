<script lang="ts">
  import { compoundStyle } from "../../lib/brands";
  import type { Units } from "../../lib/config.svelte";
  import { pressure, temp } from "../../lib/format";
  import { useLive } from "../../lib/live.svelte";
  import type { Wheel } from "../../lib/types";

  const live = useLive();

  let { opts, units }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const p = $derived(live.snap?.player);
  const me = $derived(live.snap?.vehicles.find((v) => v.isPlayer));
  const LABELS = ["FL", "FR", "RL", "RR"];

  /** Blue (cold) → green (optimal) → red (hot). */
  function tempColor(t: number, optimal: number) {
    const opt = optimal > 30 ? optimal : 90;
    const d = Math.max(-30, Math.min(30, t - opt));
    const hue = d < 0 ? 140 + (-d / 30) * 75 : 140 - (d / 30) * 140;
    return `hsl(${hue} 65% 42%)`;
  }

  function brakeColor(t: number) {
    if (t < 200) return "#3b8cff";
    if (t < 800) return "var(--color-muted)";
    if (t < 950) return "#f5a524";
    return "#ff4646";
  }

  const compound = $derived(compoundStyle(p?.frontCompound ?? ""));
</script>

{#snippet tyre(w: Wheel, i: number)}
  <div class="hud-tile p-2">
    <div class="flex items-center justify-between text-[12px]">
      <span class="font-semibold text-muted">{LABELS[i]}</span>
      <span class="font-semibold">{Math.round(w.wear * 100)}%</span>
    </div>
    <div class="mt-1.5 flex h-8 gap-0.5 overflow-hidden rounded-md" class:opacity-40={w.detached}>
      <!-- Surface temperatures as reported: left / centre / right of the tyre. -->
      {#each w.surfaceTemp as t}
        <div class="flex flex-1 items-center justify-center text-[11px] font-semibold text-white" style:background={tempColor(t, w.optimalTemp)}>
          {Math.round(temp(t, units))}
        </div>
      {/each}
    </div>
    <div class="mt-1.5 flex items-center justify-between text-[12px]">
      <span class={w.flat ? "font-bold text-bad" : "text-muted"}>
        {w.flat ? "FLAT" : pressure(w.pressure, units).toFixed(units === "imperial" ? 1 : 0)}
      </span>
      {#if opts.showBrakes}
        <span style:color={brakeColor(w.brakeTemp)}>{Math.round(temp(w.brakeTemp, units))}°</span>
      {/if}
    </div>
  </div>
{/snippet}

<div class="hud w-[260px] p-3">
  <div class="mb-2 flex items-center justify-between">
    <span class="hud-title">Tyres</span>
    {#if p?.frontCompound}
      <span class="flex items-center gap-1.5 text-[12px] text-muted">
        <span class="h-2 w-2 rounded-full" style:background={compound.color}></span>
        {p.frontCompound}{me ? ` · ${me.tyreLaps} laps` : ""}
      </span>
    {/if}
  </div>
  {#if p}
    <div class="grid grid-cols-2 gap-1.5">
      {#each p.wheels as w, i}
        {@render tyre(w, i)}
      {/each}
    </div>
  {:else}
    <div class="py-6 text-center text-sm text-muted">No car</div>
  {/if}
</div>
