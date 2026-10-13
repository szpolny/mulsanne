<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { classColor, shortName } from "../../lib/format";
  import { useLive } from "../../lib/live.svelte";
  import type { Vehicle } from "../../lib/types";
  import CarLogo from "../CarLogo.svelte";

  const live = useLive();

  let { opts }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  interface Row {
    v: Vehicle;
    /** Seconds; positive = ahead on track. */
    gap: number;
    /** +1 car is a lap (or more) ahead in the race, -1 behind, 0 same lap. */
    lapDiff: number;
  }

  const rows = $derived.by((): Row[] => {
    const snap = live.snap;
    const me = snap?.vehicles.find((v) => v.isPlayer);
    if (!snap || !me) return [];
    const len = snap.session.trackLength || 1;
    const lapTime = me.estimatedLapTime > 0 ? me.estimatedLapTime : snap.strategy.avgLapTime || 100;
    const myTotal = me.totalLaps * len + me.lapDist;

    const others: Row[] = snap.vehicles
      .filter((v) => !v.isPlayer && !v.inGarage)
      .map((v) => {
        let d = v.lapDist - me.lapDist;
        if (d > len / 2) d -= len;
        if (d < -len / 2) d += len;
        const raceDiff = v.totalLaps * len + v.lapDist - myTotal - d;
        return { v, gap: (d / len) * lapTime, lapDiff: Math.round(raceDiff / len) };
      });
    const ahead = others
      .filter((r) => r.gap >= 0)
      .sort((a, b) => a.gap - b.gap)
      .slice(0, opts.ahead ?? 3)
      .reverse();
    const behind = others
      .filter((r) => r.gap < 0)
      .sort((a, b) => b.gap - a.gap)
      .slice(0, opts.behind ?? 3);
    return [...ahead, { v: me, gap: 0, lapDiff: 0 }, ...behind];
  });

  function nameColor(r: Row) {
    if (r.v.isPlayer) return undefined;
    if (r.lapDiff > 0) return "#ff8a8a"; // a lap ahead of you
    if (r.lapDiff < 0) return "#7fb6ff"; // you are lapping them
    return undefined;
  }
</script>

<div class="hud w-[300px] py-1.5 text-[14px]">
  {#each rows as r (r.v.id)}
    <div
      class={["mx-1.5 grid h-[26px] grid-cols-[22px_36px_18px_1fr_auto_48px] items-center gap-x-2 rounded-md px-1.5", r.v.isPlayer && "bg-accent text-white"]}
    >
      <span class="text-right font-semibold">{r.v.classPlace}</span>
      <span class="rounded-[4px] text-center text-[12px] leading-[18px] font-bold text-white" style:background={classColor(r.v.class)}>{r.v.number || "–"}</span>
      <CarLogo model={r.v.model} entry={r.v.vehicle} />
      <span class="truncate font-medium" style:color={nameColor(r)}>{shortName(r.v.driver)}</span>
      <span>
        {#if r.v.inPits}<span class="rounded-[4px] bg-white px-1 text-[10px] leading-4 font-bold text-black">PIT</span>{/if}
      </span>
      <span class="text-right font-semibold">{r.v.isPlayer ? "" : (r.gap > 0 ? "−" : "+") + Math.abs(r.gap).toFixed(1)}</span>
    </div>
  {:else}
    <div class="px-3 py-2 text-muted">No cars</div>
  {/each}
</div>
