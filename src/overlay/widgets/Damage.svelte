<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { useLive } from "../../lib/live.svelte";

  const live = useLive();
  let {}: { opts: Record<string, any>; units: Units; edit: boolean } = $props();
  const p = $derived(live.snap?.player);
  const damage = $derived(p?.damage);
  const GOOD = "#43c9ad";
  const WARN = "#f5a524";
  const BAD = "#ff4646";
  const UNKNOWN = "#565963";

  // LMU dent order: nose, FL body, left side, RL body, tail, RR body, right side, FR body.
  // Group the three front/rear sections into bumpers, like the game's compact indicator.
  const BODY = [
    { label: "Front bodywork", indices: [1, 0, 7], path: "M30 29 Q30 17 56 17 Q82 17 82 29" },
    { label: "Left bodywork", indices: [2], path: "M30 38 V112" },
    { label: "Right bodywork", indices: [6], path: "M82 38 V112" },
    { label: "Rear bodywork", indices: [3, 4, 5], path: "M30 121 Q30 133 56 133 Q82 133 82 121" },
  ];
  const CORNERS = [
    { x: 43, y: 45, label: "FL" },
    { x: 69, y: 45, label: "FR" },
    { x: 43, y: 105, label: "RL" },
    { x: 69, y: 105, label: "RR" },
  ];
  const suspension = $derived(damage ? Math.max(...damage.suspension) : null);
  const dentCount = $derived(p?.dents.filter((s) => s > 0).length ?? 0);
  const detached = $derived(!!p && (p.detached || p.wheels.some((w) => w.detached)));
  const impactAge = $derived(p ? p.elapsedTime - p.lastImpactEt : -1);
  const recentImpact = $derived(!!p && p.lastImpactEt > 0 && impactAge >= 0 && impactAge < 5);
  const critical = $derived(!!p && (p.overheating || detached));
  const damaged = $derived(critical || dentCount > 0 || (damage?.aero ?? 0) > 0 || (suspension ?? 0) > 0);
  const status = $derived(!p ? "No car" : critical ? "Warning" : damaged ? "Damage" : !damage ? "Partial data" : "Normal");
  const statusColor = $derived(!p ? UNKNOWN : critical ? BAD : damaged ? WARN : !damage ? UNKNOWN : GOOD);
  const percent = (v: number | null | undefined) => v == null ? "—" : v > 0 && v < 0.01 ? "<1%" : `${Math.round(v * 100)}%`;
  const damageColor = (v: number | null | undefined) => v == null ? UNKNOWN : v >= 0.3 ? BAD : v > 0 ? WARN : GOOD;
</script>

<div class="hud damage-widget">
  <div class="heading">
    <span class="hud-title">Damage</span>
    <span class="status" style:color={statusColor}><i style:background={statusColor}></i>{status}</span>
  </div>
  <div class="content">
    <svg width="112" height="148" viewBox="0 0 112 148" role="img" aria-label="Bodywork and suspension damage, front at top">
      <title>Outer outline shows bodywork dents. Four labelled joints show suspension damage. Grey suspension means unavailable.</title>
      <path d="M52 10 L56 6 L60 10" fill="none" stroke="#9a9aa6" stroke-width="1.5" />
      <path d="M36 30 Q36 24 56 24 Q76 24 76 30 V120 Q76 126 56 126 Q36 126 36 120 Z"
        fill="rgb(255 255 255 / 0.035)" stroke="rgb(255 255 255 / 0.08)" />
      {#each BODY as zone}
        {@const severity = p ? Math.max(...zone.indices.map((i) => p.dents[i] ?? 0)) : null}
        <path d={zone.path} fill="none" stroke={severity == null || severity === 0 ? UNKNOWN : severity === 1 ? "#eac65c" : WARN}
          stroke-width="3" stroke-linecap="round">
          <title>{zone.label}: {severity == null ? "No data" : severity === 0 ? "No dents reported" : severity === 1 ? "Light dents" : "Heavy dents"}</title>
        </path>
      {/each}
      {#each CORNERS as { x, y, label }, i}
        {@const value = damage?.suspension[i]}
        <g>
          <title>{label} suspension damage: {percent(value)}</title>
          <text {x} y={y - 10} text-anchor="middle" fill="#9a9aa6" font-size="9" font-weight="600">{label}</text>
          <path d={`M${x - 6} ${y + 3} h12`} stroke={damageColor(value)} stroke-width="2" stroke-linecap="round" />
          <circle cx={x} cy={y + 3} r="3" fill="#141419" stroke={damageColor(value)} stroke-width="2" />
        </g>
      {/each}
      <text x="56" y="78" text-anchor="middle" fill="#9a9aa6" font-size="9">Susp.</text>
    </svg>
    <div class="readouts">
      <div class="readout"><span>Aero damage</span><strong style:color={damageColor(damage?.aero)}>{percent(damage?.aero)}</strong></div>
      <div class="readout" title="Most damaged suspension corner; the diagram shows each corner separately">
        <span>Suspension</span><strong style:color={damageColor(suspension)}>{percent(suspension)}</strong>
      </div>
      <div class="readout"><span>Engine temp</span><strong style:color={!p ? UNKNOWN : p.overheating ? BAD : GOOD}>{!p ? "—" : p.overheating ? "Hot" : "Normal"}</strong></div>
      <div class="readout"><span>Parts</span><strong style:color={!p ? UNKNOWN : detached ? BAD : GOOD}>{!p ? "—" : detached ? "Detached" : "Attached"}</strong></div>
      {#if !damage}<span class="unavailable">{p ? "Repair data unavailable" : "Waiting for telemetry"}</span>{/if}
    </div>
  </div>
  <div class="footer" title="Bodywork dents are separate from aerodynamic damage and can differ from the game's display.">
    <span>Bodywork <b>{p ? `${dentCount} ${dentCount === 1 ? "zone" : "zones"}` : "—"}</b></span>
    {#if recentImpact}<span class="impact">Impact detected</span>{/if}
  </div>
</div>

<style>
  .damage-widget { width: 252px; padding: 10px 12px 8px; }
  .heading { display: flex; align-items: center; justify-content: space-between; }
  .status { display: flex; align-items: center; gap: 5px; font-size: 11px; font-weight: 600; }
  .status i { width: 5px; height: 5px; border-radius: 50%; }
  .content { display: flex; align-items: center; gap: 2px; }
  svg { flex-shrink: 0; }
  .readouts { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: 12px; font-size: 11px; }
  .readout { display: flex; align-items: baseline; justify-content: space-between; gap: 6px; color: var(--color-muted); }
  .readout strong { font-size: 12px; font-weight: 600; }
  .unavailable { color: var(--color-muted); font-size: 10px; }
  .footer { display: flex; justify-content: space-between; gap: 6px; border-top: 1px solid var(--color-hud-line); padding-top: 6px; font-size: 10px; color: var(--color-muted); }
  .footer b { margin-left: 4px; color: var(--color-fg); font-weight: 500; }
  .impact { color: var(--color-warn); }
</style>
