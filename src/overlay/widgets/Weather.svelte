<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { temp, tempUnit } from "../../lib/format";
  import { live } from "../../lib/live.svelte";

  let { units }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const snap = $derived(live.snap);
  const w = $derived(snap?.session.weather);
  // Arrow points where the wind blows, relative to the car's heading (up = tailwind).
  const windAngle = $derived(w ? ((w.windDirection - (snap?.player?.yaw ?? 0)) * 180) / Math.PI : 0);

  function condition(rain: number, cloud: number) {
    if (rain > 0.6) return "Heavy rain";
    if (rain > 0.25) return "Rain";
    if (rain > 0.02) return "Light rain";
    if (cloud > 0.6) return "Overcast";
    if (cloud > 0.25) return "Cloudy";
    return "Clear";
  }
</script>

<div class="hud w-[310px] p-2.5 text-sm">
  {#if w}
    <div class="flex items-baseline justify-between">
      <span class="hud-title">{condition(w.raining, w.darkCloud)}</span>
      <span class="text-muted">Rain {Math.round(w.raining * 100)}%</span>
    </div>
    <div class="mt-2 grid grid-cols-3 gap-1.5 text-center">
      <div class="hud-tile py-1">
        <div class="hud-label">Air</div>
        <div class="font-semibold">{temp(w.ambientTemp, units).toFixed(1)}{tempUnit(units)}</div>
      </div>
      <div class="hud-tile py-1">
        <div class="hud-label">Track</div>
        <div class="font-semibold">{temp(w.trackTemp, units).toFixed(1)}{tempUnit(units)}</div>
      </div>
      <div class="flex flex-col items-center hud-tile py-1">
        <div class="hud-label">Wind</div>
        <div class="flex items-center gap-1 font-semibold">
          <svg width="12" height="12" viewBox="-6 -6 12 12" style:transform="rotate({windAngle}deg)">
            <path d="M0,-5 L3.5,3 L0,1.2 L-3.5,3Z" fill="currentColor" />
          </svg>
          {(w.windSpeed * (units === "imperial" ? 2.237 : 3.6)).toFixed(0)}
          <span class="text-[10px] text-muted">{units === "imperial" ? "mph" : "km/h"}</span>
        </div>
      </div>
    </div>
    <div class="mt-2">
      <div class="flex justify-between text-[12px]">
        <span class="text-muted">Track wetness</span>
        <span>{Math.round(w.avgWetness * 100)}% <span class="text-muted">({Math.round(w.minWetness * 100)}–{Math.round(w.maxWetness * 100)})</span></span>
      </div>
      <div class="relative mt-1 h-1.5 rounded-full bg-hud-2">
        <div class="absolute h-full rounded-full bg-accent/40" style:left="{w.minWetness * 100}%" style:width="{(w.maxWetness - w.minWetness) * 100}%"></div>
        <div class="absolute h-full w-1 rounded-full bg-accent" style:left="{w.avgWetness * 100}%"></div>
      </div>
    </div>
  {:else}
    <div class="py-3 text-center text-muted">No session</div>
  {/if}
</div>
