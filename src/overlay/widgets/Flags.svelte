<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { useLive } from "../../lib/live.svelte";

  const live = useLive();

  let { edit }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  interface Banner {
    text: string;
    sub?: string;
    bg: string;
    fg: string;
  }

  const TRACK_LIMITS_SHOW_MS = 6000;

  // Track-limit steps only accumulate, so warn when they increase rather than while non-zero.
  let lastSteps = -1;
  let trackLimitsUntil = $state(0);
  let now = $state(Date.now());

  $effect(() => {
    const steps = live.snap?.player?.trackLimitsSteps ?? -1;
    if (lastSteps >= 0 && steps > lastSteps) trackLimitsUntil = Date.now() + TRACK_LIMITS_SHOW_MS;
    lastSteps = steps;
    now = Date.now();
  });

  const view = $derived.by(() => {
    const snap = live.snap;
    const s = snap?.session;
    const p = snap?.player;
    const me = snap?.vehicles.find((v) => v.isPlayer);
    const banners: Banner[] = [];
    let lights: { total: number; lit: number } | null = null;
    if (!s) return { banners, lights };

    if (s.phase === 4 && s.numRedLights > 0) {
      lights = { total: s.numRedLights, lit: Math.min(s.startLight, s.numRedLights) };
    }
    if (s.phase === 6 || s.yellowFlagState > 0) {
      banners.push({ text: "Full course yellow", bg: "#ffd400", fg: "#111" });
    } else {
      const sectors = s.sectorYellow.flatMap((y, i) => (y ? [`S${i + 1}`] : []));
      if (sectors.length) banners.push({ text: "Yellow flag", sub: sectors.join(" · "), bg: "#ffd400", fg: "#111" });
    }
    if (s.phase === 7) banners.push({ text: "Session stopped", bg: "#e3101f", fg: "#fff" });
    if (s.phase === 8) banners.push({ text: "Chequered flag", bg: "#f4f4f6", fg: "#111" });
    if (me?.flag === 6) banners.push({ text: "Blue flag", sub: "Let faster car pass", bg: "#1d6cf2", fg: "#fff" });
    if (p && now < trackLimitsUntil) {
      const perPoint = Math.max(1, p.trackLimitsStepsPerPoint);
      banners.push({
        text: "Track limits",
        sub: `${Math.floor(p.trackLimitsSteps / perPoint)} / ${Math.round(p.trackLimitsStepsPerPenalty / perPoint)} pts`,
        bg: "#f5a524",
        fg: "#111",
      });
    }
    if (me && me.numPenalties > 0) banners.push({ text: "Penalty", sub: `×${me.numPenalties}`, bg: "#111", fg: "#fff" });
    return { banners, lights };
  });
</script>

{#if view.banners.length || view.lights || edit}
  <div class="flex w-[300px] flex-col gap-1.5">
    {#if view.lights}
      <div class="hud flex justify-center gap-2 p-2.5">
        {#each Array(view.lights.total) as _, i}
          <div class="h-7 w-7 rounded-full" style:background={i < view.lights.lit ? "#e3101f" : "rgb(255 255 255 / 0.1)"}></div>
        {/each}
      </div>
    {/if}
    {#each view.banners as b}
      <div class="flex items-baseline justify-between rounded-[10px] px-4 py-2 shadow-lg" style:background={b.bg} style:color={b.fg}>
        <span class="text-[17px] font-bold">{b.text}</span>
        {#if b.sub}<span class="text-sm font-semibold opacity-75">{b.sub}</span>{/if}
      </div>
    {/each}
    {#if edit && !view.banners.length && !view.lights}
      <div class="hud px-4 py-2 text-[17px] font-bold text-muted">Flags</div>
    {/if}
  </div>
{/if}
