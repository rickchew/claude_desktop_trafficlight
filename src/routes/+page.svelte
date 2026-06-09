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
  let label = $state("Stopped");
  let skin = $state<Skin | null>(null);
  let source = $state("");
  let lang = $state<"zh" | "en">("en");
  let showLabel = $state(true);
  let toast = $state<{kind:string;message:string}|null>(null);

  // 来源映射（按语言）
  const sourceLabels = $derived<Record<string, string>>(
    lang === "en"
      ? { files: "File watcher", process: "Subprocess", simulation: "Simulation" }
      : { files: "文件监听", process: "子进程", simulation: "模拟" }
  );

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
    // - toast 显示期间不响应
    // - 300ms 内的重复 contextmenu 忽略（macOS native menu 选择菜单项后
    //   click 漏到 webview 会重新触发 contextmenu）
    let lastMenuShown = 0;
    const onContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      if (toast) return;
      const now = performance.now();
      if (now - lastMenuShown < 300) return;
      lastMenuShown = now;
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

    // 监听语言切换
    const unlistenLang = await listen<{ lang: string }>("overlay:lang-change", (event) => {
      lang = event.payload.lang === "en" ? "en" : "zh";
    });

    // 监听 show-label 切换
    const unlistenShowLabel = await listen<{ show: boolean }>(
      "overlay:show-label-change",
      (event) => { showLabel = event.payload.show; },
    );

    // 监听通知（hooks 安装结果等）
    const unlistenNotice = await listen<{ kind: string; message: string }>(
      "overlay:notice",
      (event) => {
        toast = event.payload;
        setTimeout(() => { toast = null; }, 1800);
      },
    );

    return () => {
      document.removeEventListener("contextmenu", onContextMenu);
      unlistenState();
      unlistenSkin();
      unlistenSource();
      unlistenLang();
      unlistenShowLabel();
      unlistenNotice();
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
    {#if showLabel}
      <StatusText {label} {skin} />
    {/if}
    {#if source && sourceLabels[source]}
      <div class="source-indicator">{sourceLabels[source]}</div>
    {/if}
  </div>
  {#if toast}
    <div class="toast-overlay {toast.kind}">
      <div class="toast-symbol">{toast.kind === 'ok' ? '✓' : '✕'}</div>
    </div>
  {/if}
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

  .toast-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(4px);
    z-index: 100;
    pointer-events: none;
    animation: fadeIn 0.18s ease-out;
  }
  .toast-symbol {
    font-size: 64px;
    font-weight: 700;
    line-height: 1;
    color: #fff;
    text-shadow: 0 4px 18px currentColor;
    animation: popIn 0.32s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .toast-overlay.ok .toast-symbol {
    color: #32D74B;
  }
  .toast-overlay.err .toast-symbol {
    color: #FF453A;
  }
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes popIn {
    from { transform: scale(0.4); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
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
