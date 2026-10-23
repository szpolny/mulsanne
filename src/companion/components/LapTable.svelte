<script lang="ts">
  import { config } from "../../lib/config.svelte";
  import { fuel, fuelUnit, lapTime, sectorTime, speed, speedUnit, vePct } from "../../lib/format";
  import type { LapRecord } from "../../lib/types";
  import { uploadable } from "../../lib/uploads.svelte";
  import UploadStatus from "./UploadStatus.svelte";

  let {
    laps,
    selected = [],
    onselect,
    onopen,
    ondelete,
    fromGame = false,
  }: {
    laps: LapRecord[];
    selected?: number[];
    onselect?: (lap: LapRecord) => void;
    onopen?: (lap: LapRecord) => void;
    ondelete?: (lap: LapRecord) => void;
    /** The laps were recorded in the game (not demo data), so they can be uploaded. */
    fromGame?: boolean;
  } = $props();

  const online = $derived(config.value.online.enabled);

  const units = $derived(config.value.general.units);
  const clean = (l: LapRecord) => l.valid && !l.outLap && !l.inLap;

  const best = $derived.by(() => {
    const c = laps.filter(clean);
    const min = (f: (l: LapRecord) => number | null) => {
      const v = c.map(f).filter((x): x is number => x != null && x > 0);
      return v.length ? Math.min(...v) : null;
    };
    return { lap: min((l) => l.lapTime), s1: min((l) => l.s1), s2: min((l) => l.s2), s3: min((l) => l.s3) };
  });
  const hasVe = $derived(laps.some((l) => (l.veUsed ?? 0) > 0));
</script>

<div class="card overflow-hidden">
  <table class="w-full text-sm tnum">
    <thead class="bg-surface-2 text-left text-xs text-muted">
      <tr>
        <th class="px-3 py-2 font-medium">Lap</th>
        <th class="px-3 py-2 font-medium">Time</th>
        <th class="px-3 py-2 font-medium">S1</th>
        <th class="px-3 py-2 font-medium">S2</th>
        <th class="px-3 py-2 font-medium">S3</th>
        <th class="px-3 py-2 font-medium">Top speed</th>
        <th class="px-3 py-2 font-medium">Fuel</th>
        {#if hasVe}<th class="px-3 py-2 font-medium">Energy</th>{/if}
        <th class="px-3 py-2 font-medium"></th>
      </tr>
    </thead>
    <tbody>
      {#each laps as l (l.id)}
        {@const sel = selected.indexOf(l.id)}
        <tr
          class={[
            "border-t border-line transition-colors",
            sel >= 0 ? "bg-accent/15" : "hover:bg-surface-2",
            onselect && "cursor-pointer",
          ]}
          onclick={() => onselect?.(l)}
        >
          <td class="px-3 py-1.5">
            <div class="flex items-center gap-1.5">
              {#if sel >= 0}
                <span
                  class={["rounded px-1 text-[10px] font-bold text-white", sel === 0 ? "bg-accent" : "bg-sky-400 text-black!"]}
                  >{sel === 0 ? "A" : "B"}</span
                >
              {/if}
              {l.lapNumber}
              {#if l.outLap}<span class="rounded bg-surface-3 px-1 text-[10px] text-muted">OUT</span>{/if}
              {#if l.inLap}<span class="rounded bg-surface-3 px-1 text-[10px] text-muted">IN</span>{/if}
            </div>
          </td>
          <td
            class={[
              "px-3 py-1.5 font-semibold",
              !l.valid && "text-bad line-through decoration-1",
              clean(l) && l.lapTime === best.lap && "text-best",
            ]}>{lapTime(l.lapTime)}</td
          >
          {#each [l.s1, l.s2, l.s3] as s, i}
            <td class={["px-3 py-1.5", s != null && s === [best.s1, best.s2, best.s3][i] && "text-good"]}
              >{sectorTime(s)}</td
            >
          {/each}
          <td class="px-3 py-1.5 text-muted">{Math.round(speed(l.maxSpeed, units))} {speedUnit(units)}</td>
          <td class="px-3 py-1.5 text-muted"
            >{l.fuelUsed != null ? `${fuel(l.fuelUsed, units).toFixed(2)} ${fuelUnit(units)}` : "–"}</td
          >
          {#if hasVe}<td class="px-3 py-1.5 text-muted">{l.veUsed != null ? `${vePct(l.veUsed).toFixed(1)}%` : "–"}</td>{/if}
          <td class="px-2 py-1.5 text-right whitespace-nowrap">
            {#if online}
              <UploadStatus lapId={l.id} eligible={fromGame && uploadable(l)} />
            {/if}
            {#if onopen}
              <button
                class="rounded px-2 py-0.5 text-xs text-accent hover:bg-surface-3"
                onclick={(e) => {
                  e.stopPropagation();
                  onopen(l);
                }}>Analyse</button
              >
            {/if}
            {#if ondelete}
              <button
                class="rounded px-2 py-0.5 text-xs text-muted hover:bg-surface-3 hover:text-bad"
                title="Delete lap"
                onclick={(e) => {
                  e.stopPropagation();
                  ondelete(l);
                }}>✕</button
              >
            {/if}
          </td>
        </tr>
      {:else}
        <tr><td colspan="9" class="px-3 py-6 text-center text-muted">No laps recorded yet.</td></tr>
      {/each}
    </tbody>
  </table>
</div>
