<script lang="ts">
  import { api } from "../../lib/api";
  import { config, resetLayout, saveConfig, WIDGETS } from "../../lib/config.svelte";
  import { live } from "../../lib/live.svelte";
  import ProfileBar from "../components/ProfileBar.svelte";
  import Toggle from "../components/Toggle.svelte";
  import WidgetPreview from "../components/WidgetPreview.svelte";

  let selectedId = $state(WIDGETS[0].id);
  const meta = $derived(WIDGETS.find((w) => w.id === selectedId)!);
  const cfg = $derived(config.value.widgets[selectedId]);
  const g = $derived(config.value.general);
</script>

<div class="mx-auto max-w-6xl p-6">
  <div class="mb-5 flex items-center justify-between">
    <div>
      <h1 class="page-title">Overlays</h1>
      <p class="text-sm text-muted">Choose widgets and tune them. Position them on screen with the layout editor.</p>
    </div>
    <div class="flex gap-2">
      <button class="btn" onclick={resetLayout}>Reset positions</button>
      <button
        class={["btn", live.status.editMode ? "bg-warn! text-black" : "btn-primary"]}
        onclick={() => api.setEditMode(!live.status.editMode)}
      >
        {live.status.editMode ? "Finish editing" : "Edit layout on screen"}
      </button>
    </div>
  </div>

  <ProfileBar />

  <div class="mb-4 flex flex-wrap items-center gap-x-8 gap-y-3 card px-5 py-3 text-sm">
    <span class="text-xs font-semibold tracking-wider text-muted uppercase">All widgets</span>
    <label class="flex items-center gap-3">
      <span class="text-muted">Scale</span>
      <input type="range" min="0.4" max="3" step="0.05" bind:value={g.scale} oninput={saveConfig} class="w-40 accent-accent" />
      <span class="w-10 text-right tnum">{Math.round(g.scale * 100)}%</span>
    </label>
    <label class="flex items-center gap-3">
      <span class="text-muted">Opacity</span>
      <input type="range" min="0.2" max="1" step="0.01" bind:value={g.opacity} oninput={saveConfig} class="w-40 accent-accent" />
      <span class="w-10 text-right tnum">{Math.round(g.opacity * 100)}%</span>
    </label>
    <label class="flex items-center gap-3">
      <span class="text-muted">Background</span>
      <input type="range" min="0" max="1" step="0.01" bind:value={g.background} oninput={saveConfig} class="w-40 accent-accent" />
      <span class="w-10 text-right tnum">{Math.round(g.background * 100)}%</span>
    </label>
  </div>

  <div class="grid grid-cols-[320px_1fr] gap-4">
    <div class="flex flex-col gap-1 card p-2">
      {#each WIDGETS as w (w.id)}
        {@const wc = config.value.widgets[w.id]}
        <div
          class={[
            "flex items-center gap-3 rounded-md px-3 py-2",
            selectedId === w.id ? "bg-surface-3 shadow-[inset_3px_0_0_var(--color-accent)]" : "hover:bg-surface-2",
          ]}
        >
          <button class="min-w-0 flex-1 text-left" onclick={() => (selectedId = w.id)}>
            <div class="text-sm font-medium">{w.name}</div>
            <div class="truncate text-xs text-muted">{w.description}</div>
          </button>
          <Toggle
            checked={wc.enabled}
            label={`Enable ${w.name}`}
            onchange={(v) => {
              wc.enabled = v;
              saveConfig();
            }}
          />
        </div>
      {/each}
    </div>

    <div class="min-w-0 card p-5">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="section-title text-lg">{meta.name}</h2>
          <p class="text-sm text-muted">{meta.description}</p>
        </div>
        <Toggle
          checked={cfg.enabled}
          label="Enabled"
          onchange={(v) => {
            cfg.enabled = v;
            saveConfig();
          }}
        />
      </div>

      <div class="mt-5">
        <WidgetPreview {meta} />
      </div>

      <div class="mt-6 grid max-w-lg grid-cols-[160px_1fr_60px] items-center gap-x-4 gap-y-4 text-sm">
        <span class="text-muted">Position</span>
        <span class="col-span-2 text-muted tnum">x {Math.round(cfg.x)} · y {Math.round(cfg.y)}</span>

        <span class="text-muted">Size</span>
        <input type="range" min="0.4" max="3" step="0.05" bind:value={cfg.size} oninput={saveConfig} class="accent-accent" />
        <span class="text-right tnum">{Math.round(cfg.size * 100)}%</span>

        {#each meta.options as o (o.key)}
          <span class="text-muted">{o.label}</span>
          {#if o.type === "boolean"}
            <div class="col-span-2">
              <Toggle
                checked={!!cfg.options[o.key]}
                label={o.label}
                onchange={(v) => {
                  cfg.options[o.key] = v;
                  saveConfig();
                }}
              />
            </div>
          {:else if o.type === "number"}
            <input
              type="range"
              min={o.min}
              max={o.max}
              step={o.step}
              bind:value={cfg.options[o.key]}
              oninput={saveConfig}
              class="accent-accent"
            />
            <span class="text-right tnum">{cfg.options[o.key]}</span>
          {:else}
            <select
              class="col-span-2 field"
              bind:value={cfg.options[o.key]}
              onchange={saveConfig}
            >
              {#each o.choices as c}<option value={c.value}>{c.label}</option>{/each}
            </select>
          {/if}
        {/each}
      </div>
    </div>
  </div>
</div>
