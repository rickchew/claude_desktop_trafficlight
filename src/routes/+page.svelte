<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
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
  let menuX = $state(0);
  let menuY = $state(0);
  let skinNames = $state<string[]>([]);

  // 订阅皮肤 store
  $effect(() => {
    skin = $currentSkin;
  });

  // 右键菜单
  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    menuX = e.clientX;
    menuY = e.clientY;
    showMenu = true;

    // 拉取最新皮肤列表
    loadSkinList().then((list) => {
      skinNames = list;
    });
  }

  // 关闭菜单
  function closeMenu() {
    showMenu = false;
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

  // 切换模式 - 模拟状态（测试用）
  async function simulateState(s: string) {
    try {
      await invoke("simulate_state", { stateName: s });
    } catch (e) {
      console.error("Simulate state failed:", e);
    }
    closeMenu();
  }

  onMount(async () => {
    // 加载皮肤
    await loadCurrentSkin();
    await loadSkinList();

    // 监听状态变化
    const unlisten = await listen<StatePayload>("overlay:state-change", (event) => {
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

    // 点击外部关闭菜单
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
  class:show-menu={showMenu}
  style="
    --bg-color: {skin?.background.color ?? '#1C1C1E'};
    --bg-opacity: {skin?.background.opacity ?? 0.85};
    --border-color: {skin?.border.color ?? '#3A3A3C'};
    --border-radius: {skin?.border.radius ?? '16px'};
    --border-width: {skin?.border.width ?? '1px'};
  "
  oncontextmenu={handleContextMenu}
>
  <!-- 拖拽区域 -->
  <div class="drag-region" data-tauri-drag-region>
    <div class="traffic-light-wrapper">
      <TrafficLight {colorGroup} {animation} {skin} />
    </div>
    <StatusText {label} {skin} />
  </div>

  <!-- 右键菜单 -->
  {#if showMenu}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="context-menu"
      style="left: {menuX}px; top: {menuY}px;"
      role="menu"
    >
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
      <div class="menu-divider"></div>
      <div class="menu-header">调试</div>
      <button class="menu-item" onclick={() => simulateState("starting")} role="menuitem">启动</button>
      <button class="menu-item" onclick={() => simulateState("working")} role="menuitem">工作</button>
      <button class="menu-item" onclick={() => simulateState("thinking")} role="menuitem">思考</button>
      <button class="menu-item" onclick={() => simulateState("attention")} role="menuitem">交互</button>
      <button class="menu-item" onclick={() => simulateState("error")} role="menuitem">错误</button>
      <button class="menu-item" onclick={() => simulateState("idle")} role="menuitem">空闲</button>
      <button class="menu-item" onclick={() => simulateState("done")} role="menuitem">完成</button>
      <div class="menu-divider"></div>
      <button class="menu-item exit" onclick={handleExit} role="menuitem">退出</button>
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
    border: var(--border-width) solid var(--border-color);
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
    padding-top: 8px;
  }

  /* 右键菜单 */
  .context-menu {
    position: fixed;
    background: #2C2C2E;
    border: 1px solid #3A3A3C;
    border-radius: 12px;
    padding: 6px;
    min-width: 160px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
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
