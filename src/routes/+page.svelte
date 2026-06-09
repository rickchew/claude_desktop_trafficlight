<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import TrafficLight from "$lib/TrafficLight.svelte";
  import StatusText from "$lib/StatusText.svelte";
  import { currentSkin, loadCurrentSkin } from "$lib/SkinManager";
  import type { StatePayload, LightState, ColorGroup, AnimationType, SkinPayload, Skin } from "$lib/types";

  let state = $state<LightState>("stopped");
  let colorGroup = $state<ColorGroup>("gray");
  let animation = $state<AnimationType>("off");
  let label = $state("已停止");
  let skin = $state<Skin | null>(null);
  let source = $state("");

  // 来源中文映射
  const sourceLabels: Record<string, string> = {
    files: "文件监听",
    process: "子进程",
    simulation: "模拟",
  };

  // 订阅皮肤 store
  $effect(() => {
    skin = $currentSkin;
  });

  // 把 hex 背景色拆成 rgba 用的三分量
  const bgRgb = $derived.by(() => {
    const hex = (skin?.background.color ?? "#1C1C1E").replace("#", "");
    return {
      r: parseInt(hex.slice(0, 2), 16),
      g: parseInt(hex.slice(2, 4), 16),
      b: parseInt(hex.slice(4, 6), 16),
    };
  });

  onMount(async () => {
    // 右键点击 → 弹出原生系统菜单（切换皮肤 / 调试 / 退出）
    const onContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      invoke("show_context_menu", { x: e.screenX, y: e.screenY });
    };
    document.addEventListener("contextmenu", onContextMenu);

    await loadCurrentSkin();

    // 监听状态变化
    const unlistenState = await listen<StatePayload>("overlay:state-change", (event) => {
      state = event.payload.state;
      colorGroup = event.payload.colorGroup;
      animation = event.payload.animation;
      label = event.payload.label;
    });

    // 监听皮肤变化
    const unlistenSkin = await listen<SkinPayload>("overlay:skin-change", (event) => {
      const p = event.payload;
      skin = {
        name: p.name,
        description: p.description,
        lights: p.lights,
        background: p.background,
        border: p.border,
        label: p.label,
      };
    });

    // 监听监控模式变化
    const unlistenSource = await listen<{ source: string }>("overlay:source-change", (event) => {
      source = event.payload.source;
    });

    return () => {
      document.removeEventListener("contextmenu", onContextMenu);
      unlistenState();
      unlistenSkin();
      unlistenSource();
    };
  });
</script>

<svelte:head>
  <title>Claude Code Overlay</title>
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
</svelte:head>

<div
  class="overlay"
  style="
    --bg-r: {bgRgb.r};
    --bg-g: {bgRgb.g};
    --bg-b: {bgRgb.b};
    --bg-opacity: {skin?.background.opacity ?? 0.85};
    --border-radius: {skin?.border.radius ?? '16px'};
  "
>
  <div class="drag-region" data-tauri-drag-region>
    <div class="traffic-light-wrapper">
      <TrafficLight {colorGroup} {animation} {skin} />
    </div>
    <StatusText {label} {skin} />
    {#if source && sourceLabels[source]}
      <div class="source-indicator">{sourceLabels[source]}</div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .overlay {
    position: relative;
    width: 100vw;
    height: 100vh;
    background-image:
      linear-gradient(
        180deg,
        rgba(255, 255, 255, 0.10) 0%,
        rgba(255, 255, 255, 0.03) 18%,
        rgba(0, 0, 0, 0.18) 100%
      );
    background-color: rgba(var(--bg-r), var(--bg-g), var(--bg-b), var(--bg-opacity));
    border-radius: var(--border-radius);
    overflow: hidden;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.18),
      inset 0 -1px 0 rgba(0, 0, 0, 0.35);
  }

  .drag-region {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    cursor: grab;
  }

  .drag-region:active {
    cursor: grabbing;
  }

  .traffic-light-wrapper {
    padding-top: 0;
  }

  .source-indicator {
    font-size: 9px;
    color: var(--label-color, #EBEBF5);
    opacity: 0.4;
    text-align: center;
    padding: 2px 12px 6px;
    user-select: none;
    letter-spacing: 0.5px;
  }
</style>
