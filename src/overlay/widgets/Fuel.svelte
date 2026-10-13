<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { fuel, fuelUnit, vePct } from "../../lib/format";
  import { useLive } from "../../lib/live.svelte";

  const live = useLive();

  let { units }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const snap = $derived(live.snap);
  const p = $derived(snap?.player);
  const st = $derived(snap?.strategy);
  const hasVe = $derived((p?.virtualEnergy ?? 0) > 0);
  const n = (v: number, digits = 1) => (v > 0 && isFinite(v) ? v.toFixed(digits) : "–");
  const f = (v: number, digits = 1) => (v > 0 ? fuel(v, units).toFixed(digits) : "–");

  const fuelToAdd = $derived(st && p ? Math.max(0, st.fuelToFinish - p.fuel) : 0);
  const veToAdd = $derived(st && p ? Math.max(0, st.veToFinish - p.virtualEnergy) : 0);
  const limitingLaps = $derived.by(() => {
    if (!st) return 0;
    const vals = [st.lapsOfFuel, hasVe ? st.lapsOfVe : 0].filter((v) => v > 0);
    return vals.length ? Math.min(...vals) : 0;
  });
  const short = $derived(!!st && limitingLaps > 0 && st.lapsRemaining > limitingLaps);
  const fuelFrac = $derived(p && p.fuelCapacity > 0 ? p.fuel / p.fuelCapacity : 0);
</script>

{#snippet meter(label: string, value: string, unit: string, frac: number)}
  <div>
    <div class="flex items-baseline justify-between">
      <span class="hud-label">{label}</span>
      <span><span class="hud-value text-[22px]">{value}</span> <span class="text-[11px] text-muted">{unit}</span></span>
    </div>
    <div class="mt-1 h-1 overflow-hidden rounded-full bg-hud-2">
      <div class="h-full rounded-full bg-white/80" style:width="{Math.min(1, frac) * 100}%"></div>
    </div>
  </div>
{/snippet}

<div class="hud w-[280px] p-3 text-[14px]">
  <div class="hud-title">Fuel{hasVe ? " & energy" : ""}</div>

  <div class={["mt-2 grid gap-3", hasVe ? "grid-cols-2" : "grid-cols-1"]}>
    {@render meter("Fuel", p ? fuel(p.fuel, units).toFixed(1) : "–", fuelUnit(units), fuelFrac)}
    {#if hasVe}{@render meter("Energy", vePct(p!.virtualEnergy).toFixed(1), "%", p!.virtualEnergy)}{/if}
  </div>

  <div class="mt-3 grid grid-cols-[1fr_auto_auto] gap-x-4 gap-y-1">
    <span></span>
    <span class="hud-label text-right">Fuel</span>
    {#if hasVe}<span class="hud-label text-right">Energy</span>{:else}<span></span>{/if}

    <span class="text-muted">Per lap</span>
    <span class="text-right font-semibold">{f(st?.fuelPerLap ?? 0, 2)}</span>
    {#if hasVe}<span class="text-right font-semibold">{n(vePct(st?.vePerLap ?? 0), 2)}</span>{:else}<span></span>{/if}

    <span class="text-muted">Laps left</span>
    <span class="text-right font-semibold">{n(st?.lapsOfFuel ?? 0)}</span>
    {#if hasVe}<span class="text-right font-semibold">{n(st?.lapsOfVe ?? 0)}</span>{:else}<span></span>{/if}

    <span class="text-muted">To finish</span>
    <span class={["text-right font-semibold", fuelToAdd > 0 && "text-warn"]}>{st?.fuelPerLap ? (fuelToAdd > 0 ? `+${f(fuelToAdd)}` : "OK") : "–"}</span>
    {#if hasVe}<span class={["text-right font-semibold", veToAdd > 0 && "text-warn"]}
        >{st?.vePerLap ? (veToAdd > 0 ? `+${n(vePct(veToAdd))}` : "OK") : "–"}</span
      >{:else}<span></span>{/if}
  </div>

  <div class="mt-3 flex items-center justify-between rounded-md hud-tile px-2.5 py-1.5">
    <span class="text-muted">{snap?.session.kind === "race" ? "Race" : "Session"} laps left</span>
    <span class={["font-semibold", short && "text-bad"]}>{n(st?.lapsRemaining ?? 0)}</span>
  </div>
  {#if st && st.samples === 0}
    <div class="mt-1.5 text-[11px] text-faint">Averages appear after one clean green-flag lap.</div>
  {/if}
</div>
