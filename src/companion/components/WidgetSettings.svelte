<script lang="ts">
  import { config, saveConfig, type WidgetMeta } from "../../lib/config.svelte";
  import Select from "./Select.svelte";
  import Toggle from "./Toggle.svelte";

  /** Appearance and option controls for one widget, as label-left / control-right rows. */
  let { meta }: { meta: WidgetMeta } = $props();
  const cfg = $derived(config.value.widgets[meta.id]);
</script>

{#snippet range(value: number, min: number, max: number, step: number, text: string, oninput: (v: number) => void)}
  <span class="flex w-60 shrink-0 items-center gap-3">
    <input
      type="range"
      {min}
      {max}
      {step}
      {value}
      oninput={(e) => oninput(Number(e.currentTarget.value))}
      class="min-w-0 flex-1 accent-accent"
    />
    <span class="w-11 text-right tnum">{text}</span>
  </span>
{/snippet}

<div class="grid gap-x-10 gap-y-6 lg:grid-cols-2">
  <div>
    <h3 class="mb-1 eyebrow">Appearance</h3>
    <div class="divide-y divide-line/60 text-sm">
      <div class="flex min-h-11 items-center justify-between gap-4 py-2">
        <span class="text-soft">Size</span>
        {@render range(cfg.size, 0.4, 3, 0.05, `${Math.round(cfg.size * 100)}%`, (v) => {
          cfg.size = v;
          saveConfig();
        })}
      </div>
      <div class="flex min-h-11 items-center justify-between gap-4 py-2">
        <span class="text-soft">Opacity</span>
        {@render range(cfg.alpha, 0.2, 1, 0.01, `${Math.round(cfg.alpha * 100)}%`, (v) => {
          cfg.alpha = v;
          saveConfig();
        })}
      </div>
      <div class="flex min-h-11 items-center justify-between gap-4 py-2">
        <span class="text-soft">Position</span>
        <span class="text-faint tnum">x {Math.round(cfg.x)} · y {Math.round(cfg.y)}</span>
      </div>
    </div>
  </div>

  {#if meta.options.length}
    <div>
      <h3 class="mb-1 eyebrow">Options</h3>
      <div class="divide-y divide-line/60 text-sm">
        {#each meta.options as o (o.key)}
          <div class="flex min-h-11 items-center justify-between gap-4 py-2">
            <span class="min-w-0 text-soft">{o.label}</span>
            {#if o.type === "boolean"}
              <Toggle
                checked={!!cfg.options[o.key]}
                label={o.label}
                onchange={(v) => {
                  cfg.options[o.key] = v;
                  saveConfig();
                }}
              />
            {:else if o.type === "number"}
              {@render range(Number(cfg.options[o.key]), o.min, o.max, o.step, String(cfg.options[o.key]), (v) => {
                cfg.options[o.key] = v;
                saveConfig();
              })}
            {:else}
              <Select
                class="w-60 shrink-0"
                value={String(cfg.options[o.key])}
                label={o.label}
                options={o.choices}
                onchange={(v) => {
                  cfg.options[o.key] = v;
                  saveConfig();
                }}
              />
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
