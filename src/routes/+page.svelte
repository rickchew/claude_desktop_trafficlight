<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TrafficLight from "$lib/TrafficLight.svelte";
  import StatusText from "$lib/StatusText.svelte";
  import { currentSkin, loadCurrentSkin, loadSkinList, switchSkin } from "$lib/SkinManager";
  import type { StatePayload, LightState, ColorGroup, AnimationType, SkinPayload, Skin } from "$lib/types";

  let state = $state<LightState>("stopped");
  let colorGroup = $state<ColorGroup>("gray");
  let animation = $state<AnimationType>("off");
  let label = $state("已停止");
  let skin = $state<Skin | null>(null);
  let showMenu = $state(false);
  let skinNames = $state<string[]>([]);

  const OVERLAY_HEIGHT = 280;
  const MENU_HEIGHT = 400;

  // 订阅皮肤 store
  $effect(() => {
    skin = $currentSkin;
  });

  // 右键菜单
  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    loadSkinList().then((list) => {
      skinNames = list;
    });
    showMenu = true;
    // 拉高窗口显示完整菜单
    getCurrentWindow().setSize({ width: 130, height: MENU_HEIGHT });
  }

  // 关闭菜单
  function closeMenu() {
    showMenu = false;
    // 恢复窗口大小
    getCurrentWindow().setSize({ width: 130, height: OVERLAY_HEIGHT });
  }

  // 切换皮肤
  async function handleSwitchSkin(name: string) {
    await switchSkin(name);
    closeMenu();
  }

  // 退出应用
  async function handleExit() {
    try {
      await invoke("exit_app");
    } catch (e) {
      console.error("Exit failed:", e);
    }
  }

  // 模拟状态（测试用）
  async function simulateState(s: string) {
    try {
      await invoke("simulate_state", { stateName: s });
    } catch (e) {
      console.error("Simulate state failed:", e);
    }
    closeMenu();
  }

  onMount(async () => {
    await loadCurrentSkin();
    await loadSkinList();

    const unlisten = await listen<StatePayload>("overlay:state-change", (event) => {
      state = event.payload.state;
      colorGroup = event.payload.colorGroup;
      animation = event.payload.animation;
      label = event.payload.label;
    });

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

    const handleClick = () => {
      if (showMenu) closeMenu();
    };
    document.addEventListener("click", handleClick);

    return () => {
      unlisten();
      unlistenSkin();
      document.removeEventListener("click", handleClick);
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
  {#if showMenu}
    <!-- 全屏菜单 -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="menu-fullscreen" role="menu" onclick={(e) => e.stopPropagation()}>
      <div class="menu-scroll">
        <div class="menu-section">
          <div class="menu-header">皮肤切换</div>
          {#each skinNames as name}
            <button
              class="menu-item"
              class:active={name === skin?.name}
              onclick={() => handleSwitchSkin(name)}
              role="menuitem"
            >
              {name}
            </button>
          {/each}
        </div>
        <div class="menu-divider"></div>
        <div class="menu-section">
          <div class="menu-header">调试</div>
          <button class="menu-item" onclick={() => simulateState("starting")} role="menuitem">启动中</button>
          <button class="menu-item" onclick={() => simulateState("working")} role="menuitem">工作中</button>
          <button class="menu-item" onclick={() => simulateState("thinking")} role="menuitem">思考中</button>
          <button class="menu-item" onclick={() => simulateState("attention")} role="menuitem">需要交互</button>
          <button class="menu-item" onclick={() => simulateState("error")} role="menuitem">错误</button>
          <button class="menu-item" onclick={() => simulateState("idle")} role="menuitem">空闲</button>
          <button class="menu-item" onclick={() => simulateState("done")} role="menuitem">完成</button>
        </div>
        <div class="menu-divider"></div>
        <button class="menu-item exit" onclick={handleExit} role="menuitem">退出</button>
      </div>
    </div>
  {:else}
    <!-- 红绿灯主界面 -->
    <div class="drag-region" data-tauri-drag-region>
      <div class="traffic-light-wrapper">
        <TrafficLight {colorGroup} {animation} {skin} />
      </div>
      <StatusText {label} {skin} />
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

  /* 全屏菜单 */
  .menu-fullscreen {
    width: 100%;
    height: 100%;
    background: #2C2C2E;
    display: flex;
    flex-direction: column;
  }

  .menu-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
    -webkit-overflow-scrolling: touch;
  }

  .menu-scroll::-webkit-scrollbar {
    width: 3px;
  }

  .menu-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .menu-scroll::-webkit-scrollbar-thumb {
    background: #3A3A3C;
    border-radius: 2px;
  }

  .menu-section {
    padding: 0 4px;
  }

  .menu-header {
    padding: 6px 12px 4px;
    font-size: 10px;
    font-weight: 600;
    color: #8E8E93;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .menu-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: none;
    color: #EBEBF5;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    border-radius: 8px;
    transition: background 0.15s ease;
  }

  .menu-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .menu-item.active {
    color: #30D158;
    font-weight: 500;
  }

  .menu-item.exit {
    color: #FF453A;
  }

  .menu-divider {
    height: 1px;
    background: #3A3A3C;
    margin: 4px 8px;
  }
</style>
