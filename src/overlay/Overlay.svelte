<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import { config, loadConfig, resetLayout, switchProfile, WIDGETS } from "../lib/config.svelte";
  import { live, startLive } from "../lib/live.svelte";
  import Logo from "../lib/Logo.svelte";
  import WidgetFrame from "./WidgetFrame.svelte";
  import { COMPONENTS } from "./widgets";

  onMount(() => {
    startLive();
    loadConfig();
  });

  const edit = $derived(live.status.editMode);
  const onTrack = $derived(!!live.snap?.player && live.snap.inRealtime);
  const trackOk = $derived(!config.value.general.onlyOnTrack || onTrack);
  const focusOk = $derived(!config.value.general.hideUnfocused || live.status.gameFocused || live.status.demo);
  const show = $derived(config.loaded && (edit || (trackOk && focusOk)));
</script>

{#if edit}
  <div
    class="pointer-events-none fixed inset-0"
    style="background-image: linear-gradient(rgb(255 255 255 / 0.04) 1px, transparent 1px), linear-gradient(90deg, rgb(255 255 255 / 0.04) 1px, transparent 1px); background-size: 40px 40px; background-color: rgb(0 0 0 / 0.25);"
  ></div>
  <div
    class="hud fixed top-1/2 left-1/2 z-20 flex -translate-x-1/2 -translate-y-1/2 items-center gap-4 px-4 py-3 text-sm shadow-2xl"
  >
    <Logo size={22} />
    <div class="h-8 w-px bg-white/10"></div>
    <div>
      <div class="hud-title">Edit layout</div>
      <div class="text-xs text-muted">Drag to move · scroll to resize · <kbd class="kbd">Ctrl+Shift+F9</kbd> to finish</div>
    </div>
    {#if !live.snap?.player}
      <button class="btn px-3! py-1.5! text-xs!" onclick={() => api.setDemoMode(true)}>Demo data</button>
    {/if}
    {#if Object.keys(config.value.profiles).length > 1}
      <select
        class="field py-1.5! text-xs!"
        title="Profile"
        value={config.value.activeProfile}
        onchange={(e) => switchProfile(e.currentTarget.value)}
      >
        {#each Object.entries(config.value.profiles) as [id, p] (id)}<option value={id}>{p.name}</option>{/each}
      </select>
    {/if}
    <button class="btn px-3! py-1.5! text-xs!" onclick={resetLayout}>Reset</button>
    <button class="btn btn-primary px-4! py-1.5! text-xs!" onclick={() => api.setEditMode(false)}>Done</button>
  </div>
{/if}

{#if show}
  {#each WIDGETS as meta (meta.id)}
    {@const cfg = config.value.widgets[meta.id]}
    {#if cfg?.enabled}
      {@const Widget = COMPONENTS[meta.id]}
      <WidgetFrame id={meta.id} name={meta.name} {edit}>
        <Widget opts={cfg.options} units={config.value.general.units} {edit} />
      </WidgetFrame>
    {/if}
  {/each}
{/if}
