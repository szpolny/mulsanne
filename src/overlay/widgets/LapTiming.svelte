<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { lapTime, sectorTime } from "../../lib/format";
  import { useLive } from "../../lib/live.svelte";

  const live = useLive();

  let { opts }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const snap = $derived(live.snap);
  const t = $derived(snap?.timing);
  const p = $derived(snap?.player);

  /** Sector colour: purple = PB, green = session best, amber = slower. */
  function sectorColor(value: number, i: number): string {
    if (!t || value <= 0) return "var(--color-muted)";
    const pb = t.personalBestSectors[i];
    const sb = t.sessionBestSectors[i];
    if (pb > 0 && value <= pb + 1e-3) return "var(--color-best)";
    if (sb > 0 && value <= sb + 1e-3) return "var(--color-good)";
    return "var(--color-warn)";
  }

  // Sectors of the lap in progress, falling back to the last lap.
  const sectors = $derived.by(() => {
    if (!t) return [0, 0, 0];
    return t.currentSectors[0] > 0 ? t.currentSectors : t.lastSectors;
  });

  const tlPoints = $derived(
    p && p.trackLimitsStepsPerPoint > 0 && p.trackLimitsStepsPerPenalty > 0
      ? { have: Math.floor(p.trackLimitsSteps / p.trackLimitsStepsPerPoint), max: Math.round(p.trackLimitsStepsPerPenalty / p.trackLimitsStepsPerPoint) }
      : null,
  );

  const rows = $derived([
    { label: "Last", value: lapTime(t?.lastLapTime), color: t && !t.lastLapValid ? "var(--color-bad)" : undefined },
    { label: "Best", value: lapTime(t?.sessionBest), color: "var(--color-good)" },
    { label: "PB", value: lapTime(t?.personalBest), color: "var(--color-best)" },
    { label: "Predicted", value: lapTime(t?.predictedLap), color: undefined },
  ]);
</script>

<div class="hud w-[280px] p-3">
  <div class="flex items-center justify-between">
    <span class="hud-title">Lap {p ? p.lapNumber + 1 : "–"}</span>
    <div class="flex items-center gap-1.5">
      {#if tlPoints && tlPoints.have > 0}
        <span class="rounded-md bg-warn/20 px-1.5 text-[11px] leading-5 font-semibold text-warn" title="Track limit points">
          TL {tlPoints.have}/{tlPoints.max}
        </span>
      {/if}
      {#if p?.lapInvalidated}
        <span class="rounded-md bg-bad px-1.5 text-[11px] leading-5 font-bold text-white">Invalid</span>
      {/if}
    </div>
  </div>
  <div class={["hud-value mt-1.5 text-[38px]", p?.lapInvalidated && "text-bad"]}>{lapTime(t?.currentLapTime, 1)}</div>

  <div class="mt-2.5 flex flex-col gap-1 text-[14px]">
    {#each rows as r}
      <div class="flex justify-between">
        <span class="text-muted">{r.label}</span>
        <span class="font-semibold" style:color={r.color}>{r.value}</span>
      </div>
    {/each}
  </div>

  {#if opts.showSectors}
    <div class="mt-3 grid grid-cols-3 gap-1.5 text-center">
      {#each [0, 1, 2] as i}
        <div class="hud-tile py-1.5">
          <div class="hud-label">S{i + 1}</div>
          <div class="mt-0.5 text-[14px] font-semibold" style:color={sectorColor(sectors[i], i)}>{sectorTime(sectors[i])}</div>
        </div>
      {/each}
    </div>
  {/if}
</div>
