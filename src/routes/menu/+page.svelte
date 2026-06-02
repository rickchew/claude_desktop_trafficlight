<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import type { SkinPayload } from "$lib/types";

  let skinList = $state<string[]>([]);
  let currentSkin = $state("default");
  let label = $state("已停止");

  // 从 URL hash 读取初始数据
  function parseHash() {
    try {
      const data = JSON.parse(decodeURIComponent(window.location.hash.slice(1) || "{}"));
      skinList = data.skins || [];
      currentSkin = data.currentSkin || "default";
      label = data.label || "已停止";
    } catch {}
  }

  parseHash();

  async function handleSwitchSkin(name: string) {
    try {
      await invoke("switch_skin", { name });
      getCurrentWindow().close();
    } catch (e) {
      console.error("Switch skin failed:", e);
    }
  }

  async function handleSimulate(s: string) {
    try {
      await invoke("simulate_state", { stateName: s });
      getCurrentWindow().close();
    } catch (e) {
      console.error("Simulate state failed:", e);
    }
  }

  async function handleExit() {
    try {
      await invoke("exit_app");
    } catch (e) {
      console.error("Exit failed:", e);
    }
  }

  // 点击外部自动关闭（失焦时）
  onMount(() => {
    const win = getCurrentWindow();
    win.onFocusChanged(({ payload: focused }) => {
      if (!focused) win.close();
    });

    // 监听主窗口发来的皮肤列表更新
    const unlisten = listen<string[]>("menu:update-skins", (e) => {
      skinList = e.payload;
    });

    return () => { unlisten(); };
  });
</script>

<svelte:head>
  <title>菜单</title>
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <style>
    :global(body) {
      margin: 0;
      padding: 0;
      overflow: hidden;
      background: transparent;
      font-family: system-ui, -apple-system, sans-serif;
    }
  </style>
</svelte:head>

<div class="menu-window">
  <div class="menu-scroll">
    <div class="menu-section">
      <div class="menu-header">皮肤切换</div>
      {#each skinList as name}
        <button
          class="menu-item"
          class:active={name === currentSkin}
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
      <button class="menu-item" onclick={() => handleSimulate("starting")} role="menuitem">启动中</button>
      <button class="menu-item" onclick={() => handleSimulate("working")} role="menuitem">工作中</button>
      <button class="menu-item" onclick={() => handleSimulate("thinking")} role="menuitem">思考中</button>
      <button class="menu-item" onclick={() => handleSimulate("attention")} role="menuitem">需要交互</button>
      <button class="menu-item" onclick={() => handleSimulate("error")} role="menuitem">错误</button>
      <button class="menu-item" onclick={() => handleSimulate("idle")} role="menuitem">空闲</button>
      <button class="menu-item" onclick={() => handleSimulate("done")} role="menuitem">完成</button>
    </div>
    <div class="menu-divider"></div>
    <button class="menu-item exit" onclick={handleExit} role="menuitem">退出</button>
  </div>
</div>

<style>
  .menu-window {
    width: 100vw;
    height: 100vh;
    background: #2C2C2E;
    border: 1px solid #3A3A3C;
    border-radius: 12px;
    overflow: hidden;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
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
