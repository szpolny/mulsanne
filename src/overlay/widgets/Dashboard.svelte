<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Units } from "../../lib/config.svelte";
  import { speed, speedUnit } from "../../lib/format";
  import { useLive } from "../../lib/live.svelte";

  const live = useLive();

  let { opts, units }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const p = $derived(live.snap?.player);
  const rpmFrac = $derived(p && p.maxRpm > 0 ? Math.min(1, p.rpm / p.maxRpm) : 0);
  const gear = $derived(!p ? "–" : p.gear < 0 ? "R" : p.gear === 0 ? "N" : String(p.gear));
  // Shift light colour: green → amber → red, flashing at the limiter.
  const rpmColor = $derived(rpmFrac > 0.95 ? "#e3101f" : rpmFrac > 0.85 ? "#f5a524" : "#22c55e");
  const flash = $derived(rpmFrac > 0.97);

  const ON = "#7dd3fc";
  const WARN = "#f5a524";
  // Engine light: red when overheating, amber with the ignition off or anti-stall engaged.
  const engineColor = $derived(!p ? null : p.overheating ? "#ff4646" : p.ignition === 0 || p.antiStall ? WARN : null);

  // electricMotorState: 0 = no hybrid, 1 = idle, 2 = deploying, 3 = regenerating.
  const hybrid = $derived(!!p && p.electricMotorState > 0);
  const battery = $derived(p ? Math.max(0, Math.min(1, p.battery)) : 0);
  const motor = $derived(
    p?.electricMotorState === 3
      ? { label: "Regen", color: "var(--color-clutch)" }
      : p?.electricMotorState === 2
        ? { label: "Deploy", color: "var(--color-good)" }
        : { label: "Idle", color: "var(--color-muted)" },
  );
  const batteryColor = $derived(
    p?.electricMotorState === 3 ? "var(--color-clutch)" : battery < 0.15 ? "var(--color-bad)" : "var(--color-good)",
  );

  const ffb = $derived(p ? Math.max(-1, Math.min(1, p.ffb)) : 0);
  const clipping = $derived(Math.abs(ffb) >= 0.99);
</script>

{#snippet chip(label: string, value: string | number, active: boolean)}
  <div class={["flex items-baseline justify-between rounded-md px-2 py-1", active ? "bg-warn text-black" : "hud-tile"]}>
    <span class={["text-[11px] font-semibold", active ? "text-black/70" : "text-muted"]}>{label}</span>
    <span class="text-[14px] font-semibold">{value}</span>
  </div>
{/snippet}

{#snippet indicator(title: string, color: string | null, icon: Snippet)}
  <div
    {title}
    class={["grid h-7 place-items-center rounded-md transition-colors", color ? "bg-hud-2" : "text-faint opacity-60"]}
    style:color
  >
    <svg
      viewBox="0 0 24 24"
      class="size-5"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      {@render icon()}
    </svg>
  </div>
{/snippet}

{#snippet headlightIcon()}
  <path d="M13 5c4 0 7 3.1 7 7s-3 7-7 7c-1 0-1.5-.6-1.5-1.5v-11C11.5 5.6 12 5 13 5Z" />
  <path d="M3 7h5.5M3 10.3h5.5M3 13.7h5.5M3 17h5.5" />
{/snippet}

{#snippet limiterIcon()}
  <path d="M4.2 17a9 9 0 1 1 15.6 0" />
  <path d="m12 13 4-4.5" />
  <circle cx="12" cy="13" r="1.2" fill="currentColor" />
  <path d="M6.5 13H5M19 13h-1.5M12 6.5V5" />
{/snippet}

{#snippet tcIcon()}
  <rect x="6" y="6" width="12" height="12" rx="1.5" />
  <path d="M9.5 9.5h5M12 9.5v5.5" />
  <path d="M9 3v3M15 3v3M9 18v3M15 18v3M3 9h3M3 15h3M18 9h3M18 15h3" />
{/snippet}

{#snippet absIcon()}
  <circle cx="12" cy="12" r="6.5" />
  <path d="M4.5 6.5a9.5 9.5 0 0 0 0 11M19.5 6.5a9.5 9.5 0 0 1 0 11" />
  <text x="12" y="14.2" text-anchor="middle" font-size="5.5" font-weight="700" fill="currentColor" stroke="none">ABS</text>
{/snippet}

{#snippet wiperIcon()}
  <path d="M3 17a11 11 0 0 1 18 0" />
  <path d="M12 18 7.5 8.5" />
  <circle cx="12" cy="18" r="1" fill="currentColor" />
{/snippet}

{#snippet engineIcon()}
  <path d="M8 7h5M10.5 7v3" />
  <path d="M3 12v4M3 14h2M5 10h10l2 2h2v-1.5M21 10.5V17M21 15h-2l-2 2H8l-3-3v-4Z" />
{/snippet}

<div class="hud w-[320px] p-3">
  <div class="h-1.5 overflow-hidden rounded-full bg-hud-2" class:animate-pulse={flash}>
    <div class="h-full rounded-full transition-[width] duration-75" style:width="{rpmFrac * 100}%" style:background={rpmColor}></div>
  </div>

  <div class="mt-3 grid grid-cols-[1fr_auto_1fr] items-end">
    <div>
      <div class="hud-value text-[40px]">{p ? Math.round(speed(p.speed, units)) : "–"}</div>
      <div class="hud-label mt-1">{speedUnit(units)}</div>
    </div>
    <div class={["hud-value px-4 text-[64px] font-bold", p?.speedLimiter && "text-warn"]}>{gear}</div>
    <div class="text-right">
      <div class="hud-value text-[24px]">{p ? Math.round(p.rpm) : "–"}</div>
      <div class="hud-label mt-1">rpm</div>
    </div>
  </div>

  {#if opts.showIndicators && p}
    <div class="mt-3 grid grid-cols-6 gap-1.5">
      {@render indicator("Headlights", p.headlights ? ON : null, headlightIcon)}
      {@render indicator("Pit limiter", p.speedLimiter ? WARN : null, limiterIcon)}
      {@render indicator("Traction control", p.tcActive ? WARN : null, tcIcon)}
      {@render indicator("ABS", p.absActive ? WARN : null, absIcon)}
      {@render indicator("Wipers", p.wipers > 0 ? ON : null, wiperIcon)}
      {@render indicator("Engine", engineColor, engineIcon)}
    </div>
  {/if}

  {#if opts.showBattery && hybrid && p}
    <div class="mt-2.5 flex items-center gap-2">
      <svg viewBox="0 0 24 24" class="size-4 shrink-0 text-muted" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <rect x="2.5" y="7" width="17" height="10" rx="1.5" />
        <path d="M21.5 10.5v3" />
        <path d="m11.5 8.5-2.5 4h3l-2 3" />
      </svg>
      <div class="h-2 flex-1 overflow-hidden rounded-full bg-hud-2">
        <div class="h-full rounded-full transition-[width] duration-150" style:width="{battery * 100}%" style:background={batteryColor}></div>
      </div>
      <span class="w-9 text-right text-[13px] font-semibold tabular-nums">{Math.round(battery * 100)}%</span>
      <span class="w-11 text-right text-[10px] font-semibold tracking-wide uppercase" style:color={motor.color}>{motor.label}</span>
    </div>
  {/if}

  {#if opts.showFfb && p}
    <div class="mt-2.5 flex items-center gap-2">
      <span class={["w-7 text-[11px] font-semibold", clipping ? "text-bad" : "text-muted"]}>FFB</span>
      <div class="relative h-2 flex-1 overflow-hidden rounded-full bg-hud-2">
        <div class="absolute inset-y-0 left-1/2 w-px bg-hud-line"></div>
        <div
          class="absolute inset-y-0"
          style:left="{ffb < 0 ? 50 + ffb * 50 : 50}%"
          style:width="{Math.abs(ffb) * 50}%"
          style:background={clipping ? "var(--color-bad)" : "var(--color-steer)"}
        ></div>
      </div>
      <span class={["w-9 text-right text-[13px] font-semibold tabular-nums", clipping && "text-bad"]}>{Math.round(Math.abs(ffb) * 100)}%</span>
    </div>
  {/if}

  {#if opts.showSettings && p}
    <div class="mt-3 grid grid-cols-4 gap-1.5">
      {@render chip("TC", p.tcMax > 0 ? p.tc : "–", p.tcActive)}
      {@render chip("ABS", p.absMax > 0 ? p.abs : "–", p.absActive)}
      {@render chip("MAP", p.motorMapMax > 0 ? p.motorMap : "–", false)}
      {@render chip("BB", (p.brakeBiasFront * 100).toFixed(1), false)}
    </div>
  {/if}
</div>
