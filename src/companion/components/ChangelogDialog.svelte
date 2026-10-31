<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { config, saveConfig } from "../../lib/config.svelte";
  import { updater } from "../../lib/updater.svelte";

  let dialog: HTMLDialogElement;

  $effect(() => {
    if (updater.changelogOpen) dialog.showModal();
    else dialog.close();
  });
</script>

<dialog
  bind:this={dialog}
  aria-labelledby="changelog-title"
  class="m-auto w-[min(36rem,calc(100vw-3rem))] max-h-[calc(100vh-3rem)] overflow-hidden rounded-xl border border-line bg-surface p-0 text-fg shadow-2xl backdrop:bg-black/70"
  onclose={() => (updater.changelogOpen = false)}
>
  <div class="flex max-h-[calc(100vh-3rem-2px)] flex-col">
    <header class="flex shrink-0 items-start justify-between gap-4 border-b border-line p-6">
      <div>
        <div class="mb-1 text-xs font-semibold tracking-wider text-accent uppercase">Mulsanne · Version {updater.version}</div>
        <h2 id="changelog-title" class="page-title">What's new</h2>
      </div>
      <button class="btn" aria-label="Close changelog" onclick={() => (updater.changelogOpen = false)}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="m4 4 8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" /></svg>
      </button>
    </header>

    <div class="min-h-0 space-y-6 overflow-y-auto p-6 text-sm leading-relaxed">
      {#each updater.changelog as release}
        <div class="space-y-5">
          {#if updater.changelog.length > 1}
            <div class="flex items-baseline gap-2 border-b border-line pb-2">
              <span class="font-semibold text-fg">Version {release.version}</span>
              {#if release.date}<span class="text-xs text-muted">{release.date}</span>{/if}
            </div>
          {/if}
          {#each release.sections as section}
            <section>
              <h3 class="section-title mb-2">{section.title}</h3>
              <ul class="list-disc space-y-2 pl-5 text-muted marker:text-accent">
                {#each section.entries as entry}
                  <li>
                    {entry.text}
                    {#each entry.refs as ref}
                      <button
                        type="button"
                        class="ml-1 text-xs text-muted/70 underline-offset-2 hover:text-fg hover:underline"
                        title="View pull request {ref.label} on GitHub"
                        onclick={() => openUrl(ref.url)}>{ref.label}</button
                      >
                    {/each}
                  </li>
                {/each}
              </ul>
            </section>
          {/each}
        </div>
      {:else}
        <p class="text-muted">No changelog entries are available for this version.</p>
      {/each}
    </div>

    <footer class="flex shrink-0 flex-wrap items-center justify-between gap-4 border-t border-line p-6">
      <label class="flex cursor-pointer items-center gap-2 text-sm text-muted">
        <input
          type="checkbox"
          class="h-4 w-4 accent-accent"
          checked={!config.value.general.showChangelog}
          onchange={(event) => {
            config.value.general.showChangelog = !event.currentTarget.checked;
            saveConfig();
          }}
        />
        Don't show again
      </label>
      <button class="btn btn-primary" onclick={() => (updater.changelogOpen = false)}>Got it</button>
      <p class="w-full text-xs text-muted">You can view the changelog and change this preference in Settings.</p>
    </footer>
  </div>
</dialog>
