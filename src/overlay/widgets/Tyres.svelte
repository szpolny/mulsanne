<script lang="ts">
  import { compoundStyle } from "../../lib/brands";
  import type { Units } from "../../lib/config.svelte";
  import { pressure, temp } from "../../lib/format";
  import { useLive } from "../../lib/live.svelte";
  import type { Wheel } from "../../lib/types";

  const live = useLive();

  let { opts, units }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const p = $derived(live.snap?.player);
  const st = $derived(live.snap?.strategy);
  const me = $derived(live.snap?.vehicles.find((v) => v.isPlayer));
  const LABELS = ["FL", "FR", "RL", "RR"];
  const layer = $derived<"surface" | "core" | "both">(opts.tempLayer ?? "surface");

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

  /** Laps until the tread reaches the worn-out limit at the average rate, null without data. */
  function lapsLeft(w: Wheel, i: number) {
    const rate = st?.tyreWearPerLap?.[i] ?? 0;
    if (!st?.tyreSamples || rate <= 0) return null;
    return Math.max(0, (w.wear - (opts.wearLimit ?? 0) / 100) / rate);
  }

  const compound = $derived(compoundStyle(p?.frontCompound ?? ""));
</script>

{#snippet tyre(w: Wheel, i: number)}
  {@const rate = st?.tyreSamples ? st.tyreWearPerLap[i] : 0}
  {@const left = lapsLeft(w, i)}
  <div class="hud-tile p-2">
    <div class="flex items-center justify-between text-[12px]">
      <span class="font-semibold text-muted">{LABELS[i]}</span>
      <span class="font-semibold">{Math.round(w.wear * 100)}%</span>
    </div>
    <div class={["mt-1.5 flex gap-0.5 overflow-hidden rounded-md", layer === "both" ? "h-10" : "h-8"]} class:opacity-40={w.detached}>
      <!-- Temperatures as reported: left / centre / right of the tyre. -->
      {#each w.surfaceTemp as s, z}
        {@const t = layer === "core" ? w.innerTemp[z] : s}
        <div class="flex flex-1 flex-col items-center justify-center leading-none text-white" style:background={tempColor(t, w.optimalTemp)}>
          <span class="text-[11px] font-semibold">{Math.round(temp(t, units))}</span>
          {#if layer === "both"}
            <span class="mt-0.5 text-[9px] opacity-75" title="Core">{Math.round(temp(w.innerTemp[z], units))}</span>
          {/if}
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
    {#if opts.showWearRate}
      <div class="mt-1 flex items-center justify-between border-t border-hud-line pt-1 text-[11px]">
        <span class="text-muted" title="Tread used per lap (average on this set)">{rate > 0 ? `−${(rate * 100).toFixed(2)}%` : "–"}</span>
        <span
          class={["font-semibold", left != null && st && st.lapsRemaining > 0 && left < st.lapsRemaining && "text-warn"]}
          title="Estimated laps left on this tyre">{left == null ? "–" : left >= 100 ? "99+" : `${Math.floor(left)}`} laps</span
        >
      </div>
    {/if}
  </div>
{/snippet}

<div class="hud w-[260px] p-3">
  <div class="mb-2 flex items-center justify-between">
    <span class="hud-title">Tyres{layer === "core" ? " · core" : ""}</span>
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
    {#if opts.showWearRate && st && !st.tyreSamples}
      <div class="mt-1.5 text-[11px] text-faint">Wear rates appear after one green-flag lap on this set.</div>
    {/if}
  {:else}
    <div class="py-6 text-center text-sm text-muted">No car</div>
  {/if}
</div>
