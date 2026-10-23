<script lang="ts">
  import { api } from "../../lib/api";
  import { config } from "../../lib/config.svelte";
  import { classColor, dateTime, lapTime, sectorTime } from "../../lib/format";
  import type { BoardSummary, LeaderboardEntry } from "../../lib/types";
  import { uploads } from "../../lib/uploads.svelte";
  import { nav } from "../nav.svelte";

  const PAGE = 50;
  const SESSION_TYPES = ["Practice", "Qualifying", "Race", "Warmup", "Test Day"];

  const enabled = $derived(config.value.online.enabled);
  let boards = $state<BoardSummary[]>([]);
  let board = $state<BoardSummary | null>(null);
  let sessionType = $state("");
  /** "" for all versions. */
  let gameVersion = $state("");
  let entries = $state<LeaderboardEntry[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state("");

  /** Server ids of laps uploaded from this PC, to highlight them. */
  const mine = $derived(new Set(Object.values(uploads.byLap).map((u) => u.remoteId).filter(Boolean)));

  async function loadBoards() {
    error = "";
    try {
      boards = await api.leaderboardBoards();
      if (!board || !boards.some((b) => b.track === board!.track && b.carClass === board!.carClass)) board = boards[0] ?? null;
    } catch (e) {
      error = String(e);
    }
  }

  async function loadEntries(more = false) {
    if (!board) {
      entries = [];
      total = 0;
      return;
    }
    loading = true;
    error = "";
    try {
      const res = await api.leaderboard({
        track: board.track,
        carClass: board.carClass,
        sessionType: sessionType || null,
        gameVersion: gameVersion ? Number(gameVersion) : null,
        limit: PAGE,
        offset: more ? entries.length : 0,
      });
      entries = more ? [...entries, ...res.entries] : res.entries;
      total = res.total;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (enabled) void loadBoards();
  });

  $effect(() => {
    void board;
    void sessionType;
    void gameVersion;
    if (enabled) void loadEntries();
  });

  // A version filter only makes sense on boards that have laps from it.
  $effect(() => {
    if (gameVersion && !board?.gameVersions.includes(Number(gameVersion))) gameVersion = "";
  });
</script>

{#if !enabled}
  <div class="mx-auto max-w-3xl p-6">
    <h1 class="mb-5 page-title">Rankings</h1>
    <div class="card p-10 text-center">
      <p class="mb-4 text-muted">Compare your lap times with other drivers. Turn on online features to see the rankings.</p>
      <button class="btn btn-primary" onclick={() => (nav.page = "settings")}>Open Settings</button>
    </div>
  </div>
{:else}
  <div class="flex h-full">
    <aside class="flex w-[300px] shrink-0 flex-col border-r border-line">
      <div class="flex items-center justify-between border-b border-line p-3">
        <span class="section-title">Tracks</span>
        <button class="rounded px-2 py-0.5 text-xs text-accent hover:bg-surface-3" onclick={loadBoards}>Refresh</button>
      </div>
      <div class="flex-1 overflow-y-auto">
        {#each boards as b (b.track + b.carClass)}
          {@const active = board?.track === b.track && board?.carClass === b.carClass}
          <button
            class={["flex w-full items-center gap-3 border-b border-line px-4 py-3 text-left", active ? "bg-surface-3" : "hover:bg-surface-2"]}
            onclick={() => (board = b)}
          >
            <span class="block h-8 w-1 shrink-0 rounded-full" style:background={classColor(b.carClass)}></span>
            <span class="min-w-0 flex-1">
              <span class="block truncate font-medium">{b.track}</span>
              <span class="block text-xs text-muted">{b.carClass} · {b.drivers} {b.drivers === 1 ? "driver" : "drivers"}</span>
            </span>
            <span class="shrink-0 text-sm font-semibold text-best tnum">{lapTime(b.bestLap)}</span>
          </button>
        {:else}
          <div class="p-6 text-center text-sm text-muted">{error ? "" : "No ranked laps yet. Upload yours from Records."}</div>
        {/each}
      </div>
    </aside>

    <section class="min-w-0 flex-1 overflow-y-auto p-6">
      {#if error}
        <p class="mb-4 text-sm text-bad">{error}</p>
      {/if}
      {#if board}
        <div class="mb-4 flex items-start justify-between gap-4">
          <div>
            <h1 class="page-title">{board.track}</h1>
            <div class="text-sm text-muted">{board.carClass} · best lap per driver</div>
          </div>
          <div class="flex shrink-0 gap-2">
            <select class="field text-sm" bind:value={sessionType} aria-label="Session type">
              <option value="">All sessions</option>
              {#each SESSION_TYPES as t}<option value={t}>{t}</option>{/each}
            </select>
            {#if board.gameVersions.length}
              <select class="field text-sm" bind:value={gameVersion} aria-label="Game version">
                <option value="">All game versions</option>
                {#each board.gameVersions as v}<option value={String(v)}>Game version {v}</option>{/each}
              </select>
            {/if}
          </div>
        </div>

        <div class="card overflow-x-auto">
          <table class="w-full text-sm whitespace-nowrap tnum">
            <thead class="bg-surface-2 text-left text-xs text-muted">
              <tr>
                <th class="w-12 px-3 py-2 font-medium">#</th>
                <th class="px-3 py-2 font-medium">Driver</th>
                <th class="px-3 py-2 font-medium">Car</th>
                <th class="px-3 py-2 font-medium">Time</th>
                <th class="px-3 py-2 font-medium">Gap</th>
                <th class="hidden px-3 py-2 font-medium xl:table-cell">Sectors</th>
                <th class="px-3 py-2 font-medium">Session</th>
                <th class="px-3 py-2 font-medium">Date</th>
                <th class="px-3 py-2 font-medium"></th>
              </tr>
            </thead>
            <tbody>
              {#each entries as e (e.lapId)}
                <tr class={["border-t border-line", mine.has(e.lapId) ? "bg-accent/15" : "hover:bg-surface-2"]}>
                  <td class="px-3 py-1.5 font-semibold">{e.rank}</td>
                  <td class="px-3 py-1.5">
                    {#if e.driver}{e.driver}{:else}<span class="text-muted">Anonymous</span>{/if}
                    {#if mine.has(e.lapId)}<span class="ml-1 rounded bg-accent px-1 text-[10px] font-bold text-white">YOU</span>{/if}
                  </td>
                  <td class="px-3 py-1.5 text-muted">{e.car}</td>
                  <td class={["px-3 py-1.5 font-semibold", e.rank === 1 && "text-best"]}>{lapTime(e.lapTime)}</td>
                  <td class="px-3 py-1.5 text-muted">{e.rank === 1 ? "–" : `+${(e.lapTime - entries[0].lapTime).toFixed(3)}`}</td>
                  <td class="hidden px-3 py-1.5 text-muted xl:table-cell">{sectorTime(e.s1)} · {sectorTime(e.s2)} · {sectorTime(e.s3)}</td>
                  <td class="px-3 py-1.5 text-muted">{e.sessionType}</td>
                  <td class="px-3 py-1.5 text-xs text-muted">{dateTime(e.recordedAt)}</td>
                  <td class="px-2 py-1.5 text-right">
                    {#if e.hasTrace && !mine.has(e.lapId)}
                      <button
                        class="rounded px-2 py-0.5 text-xs text-accent hover:bg-surface-3"
                        title="Compare your best lap on this track with this one"
                        onclick={() =>
                          nav.compareWithRanked(board!.track, {
                            id: e.lapId,
                            label: `${e.driver ?? "Anonymous"} · ${e.car}`,
                            lapTime: e.lapTime,
                            s1: e.s1,
                            s2: e.s2,
                            s3: e.s3,
                          })}>Compare</button
                      >
                    {/if}
                  </td>
                </tr>
              {:else}
                <tr><td colspan="9" class="px-3 py-6 text-center text-muted">{loading ? "Loading…" : "No ranked laps for this filter."}</td></tr>
              {/each}
            </tbody>
          </table>
        </div>
        {#if entries.length < total}
          <div class="mt-3 text-center">
            <button class="btn" disabled={loading} onclick={() => loadEntries(true)}>Show more ({total - entries.length})</button>
          </div>
        {/if}
      {/if}
    </section>
  </div>
{/if}
