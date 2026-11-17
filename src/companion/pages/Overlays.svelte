<script lang="ts">
  import { api } from "../../lib/api";
  import { config, resetLayout, saveConfig, WIDGETS } from "../../lib/config.svelte";
  import { live } from "../../lib/live.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import ProfileBar from "../components/ProfileBar.svelte";
  import Toggle from "../components/Toggle.svelte";
  import WidgetIcon from "../components/WidgetIcon.svelte";
  import WidgetPreview from "../components/WidgetPreview.svelte";
  import WidgetSettings from "../components/WidgetSettings.svelte";
  import { WIDGET_GROUPS } from "../widgetGroups";

  let selectedId = $state(WIDGETS[0].id);
  const meta = $derived(WIDGETS.find((w) => w.id === selectedId)!);
  const cfg = $derived(config.value.widgets[selectedId]);
  const g = $derived(config.value.general);
</script>

<div class="page">
  <PageHeader title="Overlays" description="Choose widgets and tune them. Position them on screen with the layout editor.">
    {#snippet actions()}
      <button class="btn" onclick={resetLayout}>Reset positions</button>
      <button
        class={["btn", live.status.editMode ? "bg-warn! text-black" : "btn-primary"]}
        onclick={() => api.setEditMode(!live.status.editMode)}
      >
        {live.status.editMode ? "Finish editing" : "Edit layout on screen"}
      </button>
    {/snippet}
  </PageHeader>

  <!-- Profile and the settings that apply to every widget in it. -->
  <div class="mb-6 flex flex-wrap items-center justify-between gap-x-6 gap-y-3 border-b border-line pb-4 text-sm">
    <div class="flex items-center gap-3">
      <span class="eyebrow">Profile</span>
      <ProfileBar />
    </div>
    <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
      <label class="flex items-center gap-2" title="Scale of every widget">
        <span class="text-muted">Scale</span>
        <input type="range" min="0.4" max="3" step="0.05" bind:value={g.scale} oninput={saveConfig} class="w-24 accent-accent" />
        <span class="w-9 text-right tnum">{Math.round(g.scale * 100)}%</span>
      </label>
      <label class="flex items-center gap-2" title="Opacity of every widget">
        <span class="text-muted">Opacity</span>
        <input type="range" min="0.2" max="1" step="0.01" bind:value={g.opacity} oninput={saveConfig} class="w-24 accent-accent" />
        <span class="w-9 text-right tnum">{Math.round(g.opacity * 100)}%</span>
      </label>
    </div>
  </div>

  <div class="grid grid-cols-[230px_minmax(0,1fr)] items-start gap-6">
    <nav class="flex flex-col gap-4" aria-label="Widgets">
      {#each WIDGET_GROUPS as group (group.name)}
        <div>
          <h3 class="mb-1 px-2.5 eyebrow">{group.name}</h3>
          {#each group.widgets as w (w.id)}
            {@const on = config.value.widgets[w.id].enabled}
            {@const selected = selectedId === w.id}
            <button
              class={[
                "flex w-full items-center gap-3 rounded-md px-2.5 py-2 text-left text-sm transition-colors",
                selected ? "bg-surface-3 text-fg" : "text-soft hover:bg-surface-2 hover:text-fg",
              ]}
              aria-current={selected}
              onclick={() => (selectedId = w.id)}
            >
              <WidgetIcon id={w.id} class={["h-4 w-4 shrink-0", on ? "text-accent-hi" : "text-faint"].join(" ")} />
              <span class="min-w-0 flex-1 truncate">{w.name}</span>
              <span class={["h-1.5 w-1.5 shrink-0 rounded-full", on ? "bg-good" : "bg-surface-3"]} title={on ? "Shown" : "Hidden"}></span>
            </button>
          {/each}
        </div>
      {/each}
    </nav>

    <section class="min-w-0 card">
      <header class="flex items-center gap-4 border-b border-line px-6 py-4">
        <span
          class={[
            "flex h-11 w-11 shrink-0 items-center justify-center rounded-lg",
            cfg.enabled ? "bg-accent/15 text-accent-hi" : "bg-surface-3 text-faint",
          ]}
        >
          <WidgetIcon id={meta.id} class="h-5 w-5" />
        </span>
        <div class="min-w-0 flex-1">
          <h2 class="section-title">{meta.name}</h2>
          <p class="truncate text-sm text-soft">{meta.description}</p>
        </div>
        <label class="flex shrink-0 items-center gap-2.5 text-sm text-muted">
          {cfg.enabled ? "Shown" : "Hidden"}
          <Toggle
            checked={cfg.enabled}
            label="Enabled"
            onchange={(v) => {
              cfg.enabled = v;
              saveConfig();
            }}
          />
        </label>
      </header>
      <div class="p-6">
        <WidgetPreview {meta} />
        <div class="mt-6">
          <WidgetSettings {meta} />
        </div>
      </div>
    </section>
  </div>
</div>
