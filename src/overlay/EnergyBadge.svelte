<script lang="ts">
  /** Remaining virtual energy 0..1, null when the car has none or it is unknown. */
  let { value }: { value: number | null } = $props();

  const low = $derived(value != null && value < 0.1);
</script>

<span
  class={["inline-flex items-center justify-end gap-0.5 text-[12px]", low ? "font-bold text-warn" : "opacity-75"]}
  title="Virtual energy left"
>
  {#if value != null}
    <svg width="7" height="10" viewBox="0 0 7 10" fill="currentColor" aria-hidden="true"><path d="M4.2 0 0 5.6h2.8L2.1 10 7 3.9H4.1z" /></svg>{Math.round(value * 100)}%
  {/if}
</span>
