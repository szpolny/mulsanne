<script lang="ts">
  import Select from "../components/Select.svelte";
  import { api } from "../../lib/api";
  import { config } from "../../lib/config.svelte";
  import { classColor, classLabel, classOrder, dateTime, lapTime, sectorTime } from "../../lib/format";
  import CarRender from "../../lib/CarRender.svelte";
  import CarSilhouette from "../../lib/CarSilhouette.svelte";
  import { liveryCandidates } from "../../lib/liveries";
  import type { BoardSummary, LeaderboardEntry } from "../../lib/types";
  import TrackOutline from "../../lib/TrackOutline.svelte";
  import { uploads } from "../../lib/uploads.svelte";
  import { nav } from "../nav.svelte";

  const PAGE = 50;
  const SESSION_TYPES = ["Practice", "Qualifying", "Race", "Warmup", "Test Day"];
  const PODIUM = ["#e6b422", "#b8bcc6", "#c8834a"];

  interface TrackGroup {
    track: string;
    /** Fastest class first. */
    boards: BoardSummary[];
    drivers: number;
  }

  const enabled = $derived(config.value.online.enabled);
  let boards = $state<BoardSummary[]>([]);
  let board = $state<BoardSummary | null>(null);
  let search = $state("");
  let sessionType = $state("");
  /** "" for all versions. */
  let gameVersion = $state("");
  let entries = $state<LeaderboardEntry[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state("");

  /** Server ids of laps uploaded from this PC, to highlight them. */
  const mine = $derived(new Set(Object.values(uploads.byLap).map((u) => u.remoteId).filter(Boolean)));

  /** One entry per track (the server sends one board per track and class), sorted by name. */
  const tracks = $derived.by(() => {
    const m = new Map<string, TrackGroup>();
    for (const b of boards) {
      const g = m.get(b.track) ?? { track: b.track, boards: [], drivers: 0 };
      g.boards.push(b);
      g.drivers += b.drivers;
      m.set(b.track, g);
    }
    const out = [...m.values()];
    for (const g of out) g.boards.sort((a, b) => classOrder(a.carClass) - classOrder(b.carClass) || a.carClass.localeCompare(b.carClass));
    return out.sort((a, b) => a.track.localeCompare(b.track, undefined, { sensitivity: "base" }));
  });

  const shown = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return q ? tracks.filter((g) => g.track.toLowerCase().includes(q)) : tracks;
  });

  const group = $derived(board ? (tracks.find((g) => g.track === board!.track) ?? null) : null);

  /** The fastest three, shown large above the table. Ties can put several on one step. */
  const podium = $derived(entries.filter((e) => e.rank <= 3).slice(0, 3));

  /** The lap's livery, then plain ones of its model for laps without one or with a livery this install lacks. */
  const liveries = (e: LeaderboardEntry) => liveryCandidates(e.vehFile, e.car, board?.carClass, e.vehicleName, liveryNames);
  /** Renders the game has, to find the livery of laps uploaded with only a vehicle name. */
  let liveryNames = $state<string[]>([]);
  $effect(() => {
    api
      .carLiveries()
      .then((names) => (liveryNames = names))
      .catch(() => {});
  });

  /** "AF Corse 2026 #83:LM" -> "AF Corse 2026 #83"; nothing when it only repeats the car. */
  function teamName(e: LeaderboardEntry): string {
    const name = e.vehicleName.replace(/:[A-Z0-9]+$/i, "").trim();
    return name && name !== e.car ? name : "";
  }

  /** "LMP2"; only when a track ranks both LMP2 specs is the ELMS one called out. */
  function tabLabel(b: BoardSummary, siblings: BoardSummary[]): string {
    const label = classLabel(b.carClass);
    const clash = siblings.some((s) => s !== b && classLabel(s.carClass) === label);
    return clash && b.carClass !== label ? b.carClass.replace(/_/g, " ") : label;
  }

  function selectTrack(g: TrackGroup) {
    // Keep the class when switching tracks, if the new track ranks it.
    board = g.boards.find((b) => b.carClass === board?.carClass) ?? g.boards[0];
  }

  async function loadBoards() {
    error = "";
    try {
      boards = await api.leaderboardBoards();
      const same = board && boards.find((b) => b.track === board!.track && b.carClass === board!.carClass);
      board = same ?? (tracks[0]?.boards[0] ?? null);
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
  import PageHeader from "../components/PageHeader.svelte";
</script>

{#if !enabled}
  <div class="page">
    <PageHeader title="Rankings" description="Best laps per track and car class from other Mulsanne drivers." />
    <div class="card empty-state">
      <p class="mb-4">Compare your lap times with other drivers. Turn on online features to see the rankings.</p>
      <button class="btn btn-primary" onclick={() => (nav.page = "settings")}>Open Settings</button>
    </div>
  </div>
{:else}
  <div class="flex h-full">
    <aside class="flex w-[320px] shrink-0 flex-col border-r border-line bg-surface/40">
      <div class="flex flex-col gap-3 border-b border-line p-4">
        <div class="flex items-center justify-between">
          <span class="eyebrow">Tracks <span class="text-faint">{tracks.length || ""}</span></span>
          <button class="-my-1 btn-link text-accent-hi" onclick={loadBoards}>Refresh</button>
        </div>
        {#if tracks.length > 6}
          <input class="field w-full text-sm" type="search" placeholder="Find a track" bind:value={search} aria-label="Find a track" />
        {/if}
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        {#each shown as g (g.track)}
          {@const active = board?.track === g.track}
          <button
            class={["mb-0.5 flex w-full items-center gap-3 rounded-md px-2.5 py-2 text-left", active ? "bg-surface-3" : "hover:bg-surface-2"]}
            onclick={() => selectTrack(g)}
          >
            <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded bg-surface-2">
              <TrackOutline track={g.track} hideMissing class="h-8 w-8 {active ? 'text-accent' : 'text-fg/70'}" />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block truncate font-display text-[17px] font-semibold tracking-[0.02em] uppercase">{g.track}</span>
              <span class="mt-1 flex items-center gap-1">
                {#each g.boards as b (b.carClass)}
                  <span class="h-1.5 w-4 rounded-full" style:background={classColor(b.carClass)} title={tabLabel(b, g.boards)}></span>
                {/each}
                <span class="ml-1 text-[11px] text-muted">{g.drivers} {g.drivers === 1 ? "lap" : "laps"}</span>
              </span>
            </span>
          </button>
        {:else}
          <div class="empty-state text-sm">
            {#if error}{:else if search}No track matches “{search}”.{:else}No ranked laps yet. Upload yours from Records.{/if}
          </div>
        {/each}
      </div>
    </aside>

    <section class="min-w-0 flex-1 overflow-y-auto">
      {#if board && group}
        <div class="relative overflow-hidden border-b border-line bg-gradient-to-r from-surface-2 to-transparent px-8 pt-7">
          <TrackOutline track={board.track} hideMissing class="pointer-events-none absolute top-4 right-8 h-[calc(100%-2rem)] w-40 text-fg/15" />
          <div class="relative">
            <p class="mb-1.5 eyebrow">Rankings</p>
            <h1 class="page-title">{board.track}</h1>
            <p class="mt-1.5 text-sm text-soft">Best lap per driver</p>
          </div>
          <div class="relative mt-4 flex flex-wrap gap-1" role="tablist" aria-label="Car class">
            {#each group.boards as b (b.carClass)}
              {@const active = b.carClass === board.carClass}
              <button
                role="tab"
                aria-selected={active}
                class={["flex items-center gap-2 rounded-t-md border-x border-t px-3.5 py-2 font-display text-[15px] font-semibold tracking-[0.04em] uppercase", active ? "border-line bg-bg" : "border-transparent text-muted hover:text-fg"]}
                onclick={() => (board = b)}
              >
                <span class="h-2 w-2 rounded-full" style:background={classColor(b.carClass)}></span>
                {tabLabel(b, group.boards)}
                <span class="font-sans text-xs font-normal tracking-normal text-muted normal-case tnum">{lapTime(b.bestLap)}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="px-8 pt-6 pb-10">
          {#if error}
            <p class="mb-4 text-sm text-bad">{error}</p>
          {/if}
          <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
            <div class="text-sm text-muted">
              {board.drivers}
              {board.drivers === 1 ? "driver" : "drivers"} in {classLabel(board.carClass)}
            </div>
            <div class="flex gap-2">
              <Select
                class="min-w-40 text-sm"
                bind:value={sessionType}
                label="Session type"
                options={[{ value: "", label: "All sessions" }, ...SESSION_TYPES.map((t) => ({ value: t, label: t }))]}
              />
              {#if board.gameVersions.length}
                <Select
                  class="min-w-44 text-sm"
                  bind:value={gameVersion}
                  label="Game version"
                  options={[
                    { value: "", label: "All game versions" },
                    ...board.gameVersions.map((v) => ({ value: String(v), label: `Game version ${v}` })),
                  ]}
                />
              {/if}
            </div>
          </div>

          {#if podium.length}
            <!-- 2nd, 1st, 3rd, like a real podium. -->
            <div class="mb-4 grid grid-cols-3 items-end gap-3">
              {#each [podium[1], podium[0], podium[2]] as e, i (e?.lapId ?? `empty-${i}`)}
                {#if e}
                  {@const color = PODIUM[e.rank - 1] ?? PODIUM[2]}
                  {@const first = e.rank === 1}
                  <div
                    class={["card relative flex flex-col overflow-hidden text-center", mine.has(e.lapId) && "ring-1 ring-accent"]}
                    style:background-image="radial-gradient(ellipse 90% 70% at 50% 0%, {color}2e, transparent 70%)"
                  >
                    <!-- Big faded position numeral behind the car. -->
                    <span
                      class="pointer-events-none absolute inset-0 flex items-center justify-center font-display leading-none font-extrabold text-transparent italic select-none"
                      style:font-size={first ? "460px" : "400px"}
                      style:-webkit-text-stroke="1.5px {color}2b"
                      style:padding-bottom="0.1em"
                      aria-hidden="true">{e.rank}</span
                    >
                    <span
                      class="absolute top-3 left-3 rounded-full border px-2.5 py-0.5 font-display text-sm font-bold tracking-[0.08em] uppercase"
                      style:color={color}
                      style:border-color="{color}66"
                      style:background="{color}1a">{["1st", "2nd", "3rd"][e.rank - 1] ?? `${e.rank}th`}</span
                    >
                    <div class={["relative flex items-center justify-center px-4", first ? "h-32 pt-6" : "h-24 pt-6"]}>
                      <CarRender
                        file={liveries(e)}
                        view="front"
                        alt={e.car}
                        class="h-full w-full object-contain drop-shadow-[0_8px_12px_rgb(0_0_0/0.5)]"
                      >
                        {#snippet fallback()}<CarSilhouette class="h-2/3 w-2/3 text-muted" />{/snippet}
                      </CarRender>
                    </div>
                    <div class="relative px-4 pb-4">
                      <div class="mt-1 truncate font-semibold">
                        {#if e.driver}{e.driver}{:else}<span class="font-normal text-muted">Anonymous</span>{/if}
                      </div>
                      <div class="truncate text-xs text-muted">{e.car}{teamName(e) ? ` · ${teamName(e)}` : ""}</div>
                      <div class={["mt-1.5 font-bold tnum", first ? "text-2xl text-best" : "text-lg"]}>{lapTime(e.lapTime)}</div>
                      <div class="text-xs text-muted tnum">{first ? "Fastest lap" : `+${(e.lapTime - podium[0].lapTime).toFixed(3)}`}</div>
                    </div>
                  </div>
                {:else}
                  <div></div>
                {/if}
              {/each}
            </div>
          {/if}

          <div class="card overflow-x-auto">
            <table class="data-table whitespace-nowrap">
              <thead>
                <tr>
                  <th class="w-14 px-3 py-2 font-medium">#</th>
                  <th class="px-3 py-2 font-medium">Driver</th>
                  <th class="px-3 py-2 font-medium">Car</th>
                  <th class="px-3 py-2 text-right font-medium">Time</th>
                  <th class="px-3 py-2 text-right font-medium">Gap</th>
                  <th class="hidden px-3 py-2 font-medium 2xl:table-cell">Sectors</th>
                  <th class="px-3 py-2 font-medium">Session</th>
                  <th class="px-3 py-2 font-medium"></th>
                </tr>
              </thead>
              <tbody>
                {#each entries as e (e.lapId)}
                  {@const you = mine.has(e.lapId)}
                  <tr class={[you ? "bg-accent/15" : "hover:bg-surface-2"]}>
                    <td class="px-3 py-2">
                      {#if e.rank <= 3}
                        <span
                          class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[11px] font-bold text-black"
                          style:background={PODIUM[e.rank - 1]}>{e.rank}</span
                        >
                      {:else}
                        <span class="inline-block w-5 text-center font-semibold text-muted">{e.rank}</span>
                      {/if}
                    </td>
                    <td class="px-3 py-2 font-medium">
                      {#if e.driver}{e.driver}{:else}<span class="font-normal text-muted">Anonymous</span>{/if}
                      {#if you}<span class="ml-1 rounded bg-accent px-1 text-[10px] font-bold text-white">YOU</span>{/if}
                    </td>
                    <td class="px-3 py-1">
                      <div class="flex items-center gap-3">
                        <!-- Fixed box so rows line up while the image loads. -->
                        <span class="flex h-9 w-[84px] shrink-0 items-center justify-center">
                          <CarRender file={liveries(e)} view="thumb" alt={e.car} class="h-9 w-[84px] object-contain">
                            {#snippet fallback()}<CarSilhouette class="h-7 w-[72px] text-muted" />{/snippet}
                          </CarRender>
                        </span>
                        <span class="min-w-0">
                          <span class="block">{e.car}</span>
                          {#if teamName(e)}<span class="block text-xs text-muted">{teamName(e)}</span>{/if}
                        </span>
                      </div>
                    </td>
                    <td class={["px-3 py-2 text-right font-semibold", e.rank === 1 && "text-best"]}>{lapTime(e.lapTime)}</td>
                    <td class="px-3 py-2 text-right text-muted">{e.rank === 1 ? "–" : `+${(e.lapTime - entries[0].lapTime).toFixed(3)}`}</td>
                    <td class="hidden px-3 py-2 text-muted 2xl:table-cell">{sectorTime(e.s1)} · {sectorTime(e.s2)} · {sectorTime(e.s3)}</td>
                    <td class="px-3 py-1 text-muted">
                      <span class="block">{e.sessionType}</span>
                      <span class="block text-xs">{dateTime(e.recordedAt)}</span>
                    </td>
                    <td class="px-2 py-2 text-right">
                      {#if e.hasTrace && !you}
                        <button
                          class="btn-link text-accent-hi"
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
                  <tr><td colspan="8" class="px-3 py-8 text-center text-muted">{loading ? "Loading…" : "No ranked laps for this filter."}</td></tr>
                {/each}
              </tbody>
            </table>
          </div>
          {#if entries.length < total}
            <div class="mt-3 text-center">
              <button class="btn" disabled={loading} onclick={() => loadEntries(true)}>Show more ({total - entries.length})</button>
            </div>
          {/if}
        </div>
      {:else if error}
        <p class="pane text-sm text-bad">{error}</p>
      {/if}
    </section>
  </div>
{/if}
