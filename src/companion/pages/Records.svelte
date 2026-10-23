<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { classColor, dateTime, lapTime, sectorTime } from "../../lib/format";
  import type { PersonalBest } from "../../lib/types";
  import { config } from "../../lib/config.svelte";
  import { uploadable, uploads } from "../../lib/uploads.svelte";
  import UploadStatus from "../components/UploadStatus.svelte";
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

  const online = $derived(config.value.online.enabled);
  const eligible = (b: PersonalBest) => b.fromGame && uploadable({ valid: true, ...b });
  const toUpload = $derived(bests.filter((b) => eligible(b) && !uploads.byLap[b.lapId]));
  let uploadError = $state("");

  async function uploadAll() {
    uploadError = "";
    try {
      await api.uploadLaps(toUpload.map((b) => b.lapId));
    } catch (e) {
      uploadError = String(e);
    }
  }
</script>

<div class="mx-auto max-w-5xl p-6">
  <div class="mb-5 flex items-start justify-between gap-4">
    <div>
      <h1 class="mb-1 page-title">Personal records</h1>
      <p class="text-sm text-muted">Your best clean lap (valid, not an out/in lap) for every track and car.</p>
      {#if uploadError}<p class="mt-1 text-xs text-bad">{uploadError}</p>{/if}
    </div>
    {#if online}
      <button class="btn shrink-0" disabled={toUpload.length === 0} onclick={uploadAll}>
        {toUpload.length ? `Upload ${toUpload.length} personal best${toUpload.length === 1 ? "" : "s"}` : "Nothing to upload"}
      </button>
    {/if}
  </div>

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
              <td class="px-3 py-2 text-right whitespace-nowrap">
                {#if online}<UploadStatus lapId={b.lapId} eligible={eligible(b)} />{/if}
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
