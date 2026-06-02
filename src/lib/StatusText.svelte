<script lang="ts">
  import type { Skin } from "./types";

  let {
    label = "已停止",
    skin = null as Skin | null,
  }: {
    label: string;
    skin: Skin | null;
  } = $props();
</script>

<div
  class="status-text"
  style="
    --label-color: {skin?.label.color ?? '#EBEBF5'};
    --label-size: {skin?.label.size ?? '11px'};
    --label-font: {skin?.label.fontFamily ?? 'system-ui, -apple-system, sans-serif'};
  "
>
  <span class="label">{label}</span>
</div>

<style>
  .status-text {
    text-align: center;
    padding: 0 12px 8px;
    user-select: none;
  }

  .label {
    display: block;
    font-size: var(--label-size);
    color: var(--label-color);
    font-family: var(--label-font);
    letter-spacing: 0.3px;
    opacity: 0.8;
    transition: color 0.3s ease;
  }
</style>
