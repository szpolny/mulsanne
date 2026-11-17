<script lang="ts">
  import { classColor, classLabel, classOrder, gap, lapTime, raceTime } from "../../lib/format";
  import type { ResultEntry } from "../../lib/types";
  import CarLogo from "../../overlay/CarLogo.svelte";
  import Select from "./Select.svelte";

  let { results, complete }: { results: ResultEntry[]; /** The race was seen to the finish. */ complete: boolean } = $props();

  const classes = $derived(
    [...new Set(results.map((r) => r.carClass))].sort((a, b) => classOrder(a) - classOrder(b) || a.localeCompare(b)),
  );
  let cls = $state("");
  // Drop a class filter that no longer applies (another session was opened).
  $effect(() => {
    if (cls && !classes.includes(cls)) cls = "";
  });

  const rows = $derived(cls ? results.filter((r) => r.carClass === cls) : results);
  const fastest = $derived.by(() => {
    const t = rows.map((r) => r.bestLap).filter((x): x is number => x != null && x > 0);
    return t.length ? Math.min(...t) : null;
  });

  /** Gap to the first car shown: laps down, or time when on the same lap. */
  function gapTo(r: ResultEntry, first: ResultEntry): string {
    if (r.status === "dnf") return "DNF";
    if (r.status === "dq") return "DQ";
    if (r === first) return complete ? "Winner" : "Leader";
    const down = first.laps - r.laps;
    if (down > 0) return `+${down} lap${down === 1 ? "" : "s"}`;
    return gap(r.gap - first.gap);
  }
</script>

<div class="mb-3 flex items-center justify-between gap-4">
  <div class="flex items-center gap-3">
    <h2 class="eyebrow">Race results</h2>
    {#if !complete}
      <span class="rounded bg-surface-3 px-1.5 py-0.5 text-xs text-muted" title="The race was left before the finish, so this is the last known order."
        >Incomplete</span
      >
    {/if}
  </div>
  {#if classes.length > 1}
    <Select
      class="w-44 text-sm"
      bind:value={cls}
      label="Class"
      options={[{ value: "", label: "All classes" }, ...classes.map((c) => ({ value: c, label: classLabel(c) }))]}
    />
  {/if}
</div>
{#if !complete}
  <p class="mb-3 text-sm text-muted">The race was left before the finish, so this is the order when it was last seen.</p>
{/if}
<div class="card mb-6 overflow-hidden">
  <table class="data-table">
    <thead>
      <tr>
        <th class="w-12 px-3 py-2 font-medium">Pos</th>
        <th class="px-3 py-2 font-medium">{cls ? "Overall" : "Class"}</th>
        <th class="px-3 py-2 font-medium">Driver</th>
        <th class="px-3 py-2 font-medium">Car</th>
        <th class="px-3 py-2 text-right font-medium">Laps</th>
        <th class="px-3 py-2 font-medium">Time</th>
        <th class="px-3 py-2 font-medium">Best lap</th>
        <th class="px-3 py-2 font-medium">Gap</th>
        <th class="px-3 py-2 text-right font-medium">Pits</th>
      </tr>
    </thead>
    <tbody>
      {#each rows as r (r.position)}
        <tr class={[r.isPlayer ? "bg-accent/15 font-semibold" : "hover:bg-surface-2", r.status === "dnf" || r.status === "dq" ? "text-muted" : ""]}>
          <td class="px-3 py-1.5 font-semibold">{cls ? r.classPosition : r.position}</td>
          <td class="px-3 py-1.5 whitespace-nowrap">
            {#if cls}
              <span class="text-muted">P{r.position}</span>
            {:else}
              <span class="inline-flex items-center gap-1.5">
                <span class="h-2.5 w-1 rounded-full" style:background={classColor(r.carClass)}></span>
                <span class="text-muted">{classLabel(r.carClass)}</span>
                <span class="text-faint">P{r.classPosition}</span>
              </span>
            {/if}
          </td>
          <td class="max-w-[240px] px-3 py-1.5">
            <div class="flex items-baseline gap-2">
              {#if r.number}<span class="shrink-0 text-xs text-faint">#{r.number}</span>{/if}
              <span class="truncate">{r.driver}</span>
              {#if r.isPlayer}<span class="shrink-0 rounded bg-accent px-1 text-[10px] font-bold text-white">YOU</span>{/if}
            </div>
            {#if r.team}<div class="truncate text-xs font-normal text-faint">{r.team}</div>{/if}
          </td>
          <td class="max-w-[220px] px-3 py-1.5">
            <span class="flex items-center gap-1.5 font-normal text-soft">
              <CarLogo model={r.car} entry={r.vehicleName} size={15} />
              <span class="truncate">{r.car || r.vehicleName}</span>
            </span>
          </td>
          <td class="px-3 py-1.5 text-right">{r.laps}</td>
          <td class="px-3 py-1.5 whitespace-nowrap">{raceTime(r.raceTime)}</td>
          <td class={["px-3 py-1.5", r.bestLap != null && r.bestLap === fastest && "text-best"]}>{lapTime(r.bestLap)}</td>
          <td class={["px-3 py-1.5 whitespace-nowrap", (r.status === "dnf" || r.status === "dq") && "text-bad"]}>{gapTo(r, rows[0])}</td>
          <td class="px-3 py-1.5 text-right text-muted">{r.pitStops}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
