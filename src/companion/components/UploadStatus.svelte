<script lang="ts">
  import { api } from "../../lib/api";
  import { uploads } from "../../lib/uploads.svelte";

  /** Upload state of one lap, with Upload / Retry / Remove actions. */
  let { lapId, eligible }: { lapId: number; eligible: boolean } = $props();

  const current = $derived(uploads.byLap[lapId]);
  let busy = $state(false);
  let error = $state("");

  async function act(action: () => Promise<unknown>) {
    busy = true;
    error = "";
    try {
      await action();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const upload = () =>
    act(async () => {
      if ((await api.uploadLaps([lapId])) === 0) error = "This lap can't be uploaded.";
    });
  const remove = () => act(() => api.removeLapUpload(lapId));

  const btn = "rounded px-2 py-0.5 text-xs hover:bg-surface-3 disabled:opacity-50";
</script>

<span class="inline-flex items-center gap-1 text-xs whitespace-nowrap" title={error || current?.error || undefined}>
  {#if !current}
    {#if eligible}
      <button class={[btn, "text-accent"]} disabled={busy} onclick={(e) => { e.stopPropagation(); upload(); }}>Upload</button>
    {/if}
  {:else if current.status === "queued"}
    <span class="px-2 text-muted">Uploading…</span>
  {:else if current.status === "ranked" || current.status === "held"}
    <span class={["px-2", current.status === "ranked" ? "text-good" : "text-muted"]}
      title={current.status === "held" ? "Hidden until reviewed: it's much faster than the current record, or its track length doesn't match other drivers'." : undefined}
      >{current.status === "ranked" ? "Ranked" : "Under review"}</span
    >
    {#if current.trace === "uploaded"}
      <span class="text-muted" title="Telemetry uploaded; others can compare against this lap.">+ telemetry</span>
    {:else if current.trace === "queued"}
      <span class="text-muted">Sending telemetry…</span>
    {:else if current.trace === "rejected"}
      <span class="text-bad" title={current.error ?? undefined}>Telemetry rejected</span>
    {/if}
    <button class={[btn, "text-muted hover:text-bad"]} disabled={busy} title="Remove from rankings"
      onclick={(e) => { e.stopPropagation(); remove(); }}>Remove</button
    >
  {:else}
    <span class="px-2 text-bad">{current.status === "failed" ? "Upload failed" : "Rejected"}</span>
    <button class={[btn, "text-accent"]} disabled={busy} onclick={(e) => { e.stopPropagation(); upload(); }}>Retry</button>
  {/if}
  {#if error}
    <span class="text-bad">!</span>
  {/if}
</span>
