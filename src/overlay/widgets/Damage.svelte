<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { live } from "../../lib/live.svelte";

  let {}: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const p = $derived(live.snap?.player);

  // The SDK only says "8 locations around the car"; we assume clockwise from the nose.
  // Each zone is an arc segment of the outline below, in SVG coordinates of a 60×110 car.
  const ZONES: [number, number][] = [
    [30, 4], // front
    [54, 22], // front right
    [57, 55], // right
    [54, 90], // rear right
    [30, 106], // rear
    [6, 90], // rear left
    [3, 55], // left
    [6, 22], // front left
  ];
  const WHEELS: [number, number][] = [
    [0, 22],
    [52, 22],
    [0, 76],
    [52, 76],
  ];
  const sevColor = (s: number) => (s >= 2 ? "#e5484d" : s === 1 ? "#f2a33a" : "rgb(255 255 255 / 0.12)");
  const recentImpact = $derived(!!p && p.lastImpactEt > 0 && p.elapsedTime - p.lastImpactEt < 5);
</script>

<div class="hud flex w-[190px] items-center gap-3 p-2.5">
  <svg width="62" height="112" viewBox="-1 -1 62 112">
    <rect x="8" y="6" width="44" height="98" rx="16" fill="rgb(255 255 255 / 0.06)" stroke="rgb(255 255 255 / 0.2)" />
    {#each WHEELS as [x, y], i}
      {@const w = p?.wheels[i]}
      <rect
        {x}
        {y}
        width="8"
        height="16"
        rx="2"
        fill={w?.detached ? "#e5484d" : w?.flat ? "#f2a33a" : "rgb(255 255 255 / 0.35)"}
      />
    {/each}
    {#each ZONES as [x, y], i}
      <circle cx={x} cy={y} r="6" fill={sevColor(p?.dents[i] ?? 0)} />
    {/each}
  </svg>
  <div class="flex flex-col gap-1 text-[12px]">
    <span class="hud-title">Damage</span>
    <span class:text-bad={p?.overheating} class:text-muted={!p?.overheating}>
      {p?.overheating ? "Overheating" : "Engine OK"}
    </span>
    <span class:text-bad={p?.detached} class:text-muted={!p?.detached}>{p?.detached ? "Parts detached" : "Aero OK"}</span>
    <span class:text-warn={recentImpact} class:text-muted={!recentImpact}>
      Impact {p && p.lastImpactMagnitude > 0 ? Math.round(p.lastImpactMagnitude) : "–"}
    </span>
  </div>
</div>
