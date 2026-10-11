<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { classColor, gap, lapTime, shortName } from "../../lib/format";
  import { live } from "../../lib/live.svelte";
  import type { Vehicle } from "../../lib/types";
  import CarLogo from "../CarLogo.svelte";
  import TyreBadge from "../TyreBadge.svelte";

  let { opts }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const snap = $derived(live.snap);
  const isRace = $derived(snap?.session.kind === "race");

  interface Row {
    v: Vehicle;
    gap: string;
    interval: string;
  }

  const groups = $derived.by(() => {
    if (!snap) return [];
    const myClass = snap.vehicles.find((v) => v.isPlayer)?.class;
    const byClass = new Map<string, Vehicle[]>();
    for (const v of [...snap.vehicles].sort((a, b) => a.place - b.place)) {
      if (opts.classes === "mine" && myClass != null && v.class !== myClass) continue;
      if (!byClass.has(v.class)) byClass.set(v.class, []);
      byClass.get(v.class)!.push(v);
    }
    const maxRows = Math.max(1, opts.maxRows ?? 8);
    return [...byClass.entries()].map(([cls, cars]) => {
      const leader = cars[0];
      const rows: Row[] = cars.map((v, i) => {
        if (i === 0) return { v, gap: isRace ? "Leader" : lapTime(v.bestLap), interval: "" };
        const prev = cars[i - 1];
        if (!isRace) {
          const d = v.bestLap > 0 && leader.bestLap > 0 ? v.bestLap - leader.bestLap : 0;
          const di = v.bestLap > 0 && prev.bestLap > 0 ? v.bestLap - prev.bestLap : 0;
          return { v, gap: d > 0 ? `+${d.toFixed(3)}` : "–", interval: di > 0 ? `+${di.toFixed(3)}` : "" };
        }
        return {
          v,
          gap: gap(v.timeBehindLeader - leader.timeBehindLeader, v.lapsBehindLeader - leader.lapsBehindLeader),
          interval: gap(v.timeBehindLeader - prev.timeBehindLeader, v.lapsBehindLeader - prev.lapsBehindLeader),
        };
      });
      // Keep the player visible: top rows plus the cars around the player.
      let visible = rows.slice(0, maxRows);
      const pi = rows.findIndex((r) => r.v.isPlayer);
      if (pi >= maxRows) {
        const top = Math.max(1, maxRows - 3);
        visible = [...rows.slice(0, top), ...rows.slice(pi - 1, pi + 2)].slice(0, maxRows);
      }
      return { cls, count: cars.length, rows: visible };
    });
  });

  const columns = $derived(
    [
      "22px", // position
      "36px", // number
      opts.showLogo ? "18px" : "",
      "minmax(130px,1fr)", // driver
      opts.showTyres ? "58px" : "",
      opts.showLast ? "64px" : "",
      "64px", // gap
      opts.showInterval ? "56px" : "",
    ]
      .filter(Boolean)
      .join(" "),
  );
</script>

<div class="hud w-max min-w-[380px] py-1.5 text-[14px]">
  {#each groups as g (g.cls)}
    <div class="flex items-center gap-2 px-3 pt-1.5 pb-1">
      <span class="h-2.5 w-2.5 rounded-full" style:background={classColor(g.cls)}></span>
      <span class="hud-title text-fg!">{g.cls || "Class"}</span>
      <span class="text-[12px] text-faint">{g.count}</span>
    </div>
    {#each g.rows as r, i (r.v.id)}
      <div
        class={[
          "mx-1.5 grid h-[26px] items-center gap-x-2 rounded-md px-1.5",
          r.v.isPlayer ? "bg-accent text-white" : i % 2 === 1 && "bg-white/[0.03]",
        ]}
        style:grid-template-columns={columns}
      >
        <span class="text-right font-semibold">{r.v.classPlace}</span>
        <span
          class="rounded-[4px] text-center text-[12px] leading-[18px] font-bold text-white"
          style:background={classColor(r.v.class)}
          style:box-shadow={r.v.isPlayer ? "0 0 0 1px rgb(255 255 255 / 0.6)" : undefined}>{r.v.number || "–"}</span
        >
        {#if opts.showLogo}<CarLogo model={r.v.model} entry={r.v.vehicle} />{/if}
        <span class="flex min-w-0 items-center gap-1.5">
          <span class="truncate font-medium">{shortName(r.v.driver)}</span>
          {#if r.v.inPits}<span class="rounded-[4px] bg-white px-1 text-[10px] leading-4 font-bold text-black">PIT</span>{/if}
        </span>
        {#if opts.showTyres}
          <TyreBadge compound={r.v.tyreCompound} wear={r.v.tyreWear} laps={r.v.tyreLaps} show={opts.tyreInfo} />
        {/if}
        {#if opts.showLast}<span class="text-right text-[13px] opacity-75">{lapTime(r.v.lastLap)}</span>{/if}
        <span class="text-right font-medium">{r.gap}</span>
        {#if opts.showInterval}<span class="text-right text-[13px] opacity-65">{r.interval}</span>{/if}
      </div>
    {/each}
  {:else}
    <div class="px-3 py-2 text-muted">No session</div>
  {/each}
</div>
