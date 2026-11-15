<script lang="ts">
  let {
    value,
    fallback,
    label,
    onchange,
  }: {
    value: string;
    /** Shortcut restored by the reset button. */
    fallback: string;
    label: string;
    /** Applies the new shortcut; rejects with a message when it can't be used. */
    onchange: (accelerator: string) => Promise<void>;
  } = $props();

  let recording = $state(false);
  let error = $state("");

  const MODIFIERS = ["Control", "Shift", "Alt", "Meta", "OS"];

  // KeyboardEvent.code → key name understood by the global shortcut parser.
  function keyName(code: string) {
    if (/^Key[A-Z]$/.test(code)) return code.slice(3);
    if (/^Digit\d$/.test(code)) return code.slice(5);
    return code;
  }

  async function apply(accelerator: string) {
    error = "";
    try {
      await onchange(accelerator);
    } catch (e) {
      error = String(e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    if (MODIFIERS.includes(e.key)) return;
    recording = false;
    if (e.key === "Escape" && !(e.ctrlKey || e.altKey || e.shiftKey || e.metaKey)) return;
    const parts = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(Boolean);
    apply([...parts, keyName(e.code)].join("+"));
  }
</script>

<div class="flex flex-col items-end gap-1">
  <div class="flex items-center gap-2">
    <button
      class={["btn min-w-40 justify-center font-mono", recording && "ring-2 ring-accent"]}
      aria-label={label}
      onclick={() => {
        error = "";
        recording = !recording;
      }}
      onblur={() => (recording = false)}
      {onkeydown}
    >
      {recording ? "Press keys…" : value}
    </button>
    <button class="btn" disabled={value === fallback} onclick={() => apply(fallback)}>Reset</button>
  </div>
  {#if error}
    <div class="max-w-72 text-right text-xs text-bad" role="alert">{error}</div>
  {/if}
</div>
