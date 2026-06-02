<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import TrafficLight from "$lib/TrafficLight.svelte";
  import StatusText from "$lib/StatusText.svelte";
  import { currentSkin, loadCurrentSkin, loadSkinList } from "$lib/SkinManager";
  import type { StatePayload, LightState, ColorGroup, AnimationType, SkinPayload, Skin } from "$lib/types";

  let state = $state<LightState>("stopped");
  let colorGroup = $state<ColorGroup>("gray");
  let animation = $state<AnimationType>("off");
  let label = $state("已停止");
  let skin = $state<Skin | null>(null);
  let skinList = $state<string[]>([]);
  let menuWindow: WebviewWindow | null = null;

  // 订阅皮肤 store
  $effect(() => {
    skin = $currentSkin;
  });

  // 右键点击 → 弹出浮动菜单窗口
  async function handleContextMenu(e: MouseEvent) {
    e.preventDefault();

    // 如果已有菜单窗口，先关闭
    if (menuWindow) {
      try { menuWindow.close(); } catch {}
      menuWindow = null;
    }

    // 获取最新皮肤列表
    const list: string[] = await invoke("list_skins");
    skinList = list;

    // 构建菜单窗口 URL（通过 hash 传递数据）
    const data = encodeURIComponent(JSON.stringify({
      skins: list,
      currentSkin: skin?.name || "default",
      label,
    }));
    const url = `/menu#${data}`;

    // 计算菜单位置（在鼠标附近，但确保不超出屏幕）
    const winWidth = 180;
    const winHeight = 380;
    const screenW = window.screen.availWidth;
    const screenH = window.screen.availHeight;
    let x = e.screenX;
    let y = e.screenY;
    if (x + winWidth > screenW) x = screenW - winWidth - 10;
    if (y + winHeight > screenH) y = screenH - winHeight - 10;
    if (x < 0) x = 10;
    if (y < 0) y = 10;

    menuWindow = new WebviewWindow("context-menu", {
      url,
      width: winWidth,
      height: winHeight,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      resizable: false,
      x,
      y,
      focus: true,
      visible: true,
    });

    // 窗口创建失败时清理
    menuWindow.once("tauri://error", () => {
      menuWindow = null;
    });

    // 窗口关闭时清理引用
    menuWindow.once("tauri://close-requested", () => {
      menuWindow = null;
    });
  }

  onMount(async () => {
    await loadCurrentSkin();
    await loadSkinList();

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

    return () => {
      unlistenState();
      unlistenSkin();
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
    --bg-color: {skin?.background.color ?? '#1C1C1E'};
    --bg-opacity: {skin?.background.opacity ?? 0.85};
    --border-radius: {skin?.border.radius ?? '16px'};
  "
  oncontextmenu={handleContextMenu}
>
  <!-- 红绿灯主界面 -->
  <div class="drag-region" data-tauri-drag-region>
    <div class="traffic-light-wrapper">
      <TrafficLight {colorGroup} {animation} {skin} />
    </div>
    <StatusText {label} {skin} />
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
    background: var(--bg-color);
    opacity: var(--bg-opacity);
    border-radius: var(--border-radius);
    overflow: hidden;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
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
    padding-top: 4px;
  }
</style>
