<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { classColor, dateTime, lapTime, sectorTime } from "../../lib/format";
  import type { PersonalBest } from "../../lib/types";
  import { nav } from "../nav.svelte";

  let bests = $state<PersonalBest[]>([]);
  onMount(async () => {
    bests = await api.personalBests();
  });

  const byTrack = $derived.by(() => {
    const m = new Map<string, PersonalBest[]>();
    for (const b of bests) {
      if (!m.has(b.track)) m.set(b.track, []);
      m.get(b.track)!.push(b);
    }
    return [...m.entries()];
  });
</script>

<div class="mx-auto max-w-5xl p-6">
  <h1 class="mb-1 page-title">Personal records</h1>
  <p class="mb-5 text-sm text-muted">Your best clean lap (valid, not an out/in lap) for every track and car.</p>

  {#each byTrack as [track, rows] (track)}
    <div class="card mb-5 overflow-hidden">
      <div class="section-title bg-surface-2 px-4 py-2.5">{track}</div>
      <table class="w-full text-sm tnum">
        <tbody>
          {#each rows as b (b.lapId)}
            <tr class="border-t border-line hover:bg-surface-2">
              <td class="w-2 py-2 pl-4"><span class="block h-4 w-1 rounded-full" style:background={classColor(b.carClass)}></span></td>
              <td class="px-3 py-2">
                <div>{b.car}</div>
                <div class="text-xs text-muted">{b.carClass} · {b.lapCount} clean laps</div>
              </td>
              <td class="px-3 py-2 text-lg font-semibold text-best">{lapTime(b.lapTime)}</td>
              <td class="px-3 py-2 text-muted">{sectorTime(b.s1)} · {sectorTime(b.s2)} · {sectorTime(b.s3)}</td>
              <td class="px-3 py-2 text-xs text-muted">{dateTime(b.recordedAt)}</td>
              <td class="px-3 py-2 text-right">
                <button class="rounded px-2 py-1 text-xs text-accent hover:bg-surface-3" onclick={() => nav.openAnalysis(b.track, b.lapId)}>Analyse</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="card p-10 text-center text-muted">No clean laps recorded yet.</div>
  {/each}
</div>
