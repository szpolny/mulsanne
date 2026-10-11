<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { speed, speedUnit } from "../../lib/format";
  import { live } from "../../lib/live.svelte";

  let { opts, units }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const p = $derived(live.snap?.player);
  const rpmFrac = $derived(p && p.maxRpm > 0 ? Math.min(1, p.rpm / p.maxRpm) : 0);
  const gear = $derived(!p ? "–" : p.gear < 0 ? "R" : p.gear === 0 ? "N" : String(p.gear));
  // Shift light colour: green → amber → red, flashing at the limiter.
  const rpmColor = $derived(rpmFrac > 0.95 ? "#e3101f" : rpmFrac > 0.85 ? "#f5a524" : "#22c55e");
  const flash = $derived(rpmFrac > 0.97);
</script>

{#snippet bar(label: string, value: number, color: string)}
  <div class="flex items-center gap-2">
    <span class="w-3 text-[11px] font-semibold text-muted">{label}</span>
    <div class="h-1.5 flex-1 overflow-hidden rounded-full bg-hud-2">
      <div class="h-full rounded-full" style:background={color} style:width="{value * 100}%"></div>
    </div>
  </div>
{/snippet}

{#snippet chip(label: string, value: string | number, active: boolean)}
  <div class={["flex items-baseline justify-between rounded-md px-2 py-1", active ? "bg-warn text-black" : "hud-tile"]}>
    <span class={["text-[11px] font-semibold", active ? "text-black/70" : "text-muted"]}>{label}</span>
    <span class="text-[14px] font-semibold">{value}</span>
  </div>
{/snippet}

<div class="hud w-[320px] p-3">
  <div class="h-1.5 overflow-hidden rounded-full bg-hud-2" class:animate-pulse={flash}>
    <div class="h-full rounded-full transition-[width] duration-75" style:width="{rpmFrac * 100}%" style:background={rpmColor}></div>
  </div>

  <div class="mt-3 grid grid-cols-[1fr_auto_1fr] items-end">
    <div>
      <div class="hud-value text-[40px]">{p ? Math.round(speed(p.speed, units)) : "–"}</div>
      <div class="hud-label mt-1">{speedUnit(units)}</div>
    </div>
    <div class={["hud-value px-4 text-[64px] font-bold", p?.speedLimiter && "text-warn"]}>{gear}</div>
    <div class="text-right">
      <div class="hud-value text-[24px]">{p ? Math.round(p.rpm) : "–"}</div>
      <div class="hud-label mt-1">rpm</div>
    </div>
  </div>

  {#if opts.showPedals}
    <div class="mt-3 flex flex-col gap-1.5">
      {@render bar("T", p?.throttle ?? 0, "var(--color-throttle)")}
      {@render bar("B", p?.brake ?? 0, p?.absActive ? "var(--color-warn)" : "var(--color-brake)")}
      {@render bar("C", p?.clutch ?? 0, "var(--color-clutch)")}
    </div>
  {/if}

  {#if opts.showSettings && p}
    <div class="mt-3 grid grid-cols-4 gap-1.5">
      {@render chip("TC", p.tcMax > 0 ? p.tc : "–", p.tcActive)}
      {@render chip("ABS", p.absMax > 0 ? p.abs : "–", p.absActive)}
      {@render chip("MAP", p.motorMapMax > 0 ? p.motorMap : "–", false)}
      {@render chip("BB", (p.brakeBiasFront * 100).toFixed(1), false)}
    </div>
  {/if}
</div>
