<script lang="ts">
  import type { ColorGroup, AnimationType, LightConfig, Skin } from "./types";

  let {
    colorGroup = "gray" as ColorGroup,
    animation = "off" as AnimationType,
    skin = null as Skin | null,
  }: {
    colorGroup: ColorGroup;
    animation: AnimationType;
    skin: Skin | null;
  } = $props();

  // 获取灯配置
  function getLightConfig(group: ColorGroup): LightConfig | null {
    if (!skin) return null;
    switch (group) {
      case "red": return skin.lights.red;
      case "yellow": return skin.lights.yellow;
      case "green": return skin.lights.green;
      case "gray": return skin.lights.gray;
    }
  }

  // 获取灯亮灭状态
  function isActive(group: ColorGroup): boolean {
    if (animation === "off") return false;
    return group === colorGroup;
  }

  function getLightColor(group: ColorGroup): string {
    const cfg = getLightConfig(group);
    if (!cfg) return "#3A3A3C";
    return isActive(group) ? cfg.on : cfg.off;
  }

  function getGlow(group: ColorGroup): string {
    const cfg = getLightConfig(group);
    if (!cfg || !cfg.glow || animation === "off") return "none";
    return isActive(group) ? cfg.glow : "none";
  }

  // 灯序号和对应的颜色组
  const lights: { index: number; group: ColorGroup }[] = [
    { index: 0, group: "red" },
    { index: 1, group: "yellow" },
    { index: 2, group: "green" },
  ];
</script>

<div class="traffic-light">
  {#each lights as { index, group } (index)}
    <div
      class="light-container"
      class:active={isActive(group)}
      class:blink={animation === "slow-blink" || animation === "fast-blink"}
      class:breathing={animation === "breathing"}
      style="
        --light-color: {getLightColor(group)};
        --light-glow: {getGlow(group)};
        --blink-speed: {animation === 'fast-blink' ? '0.3s' : animation === 'slow-blink' ? '1s' : '0s'};
      "
    >
      <div class="light">
        <div class="light-inner"></div>
        <div class="light-reflect"></div>
      </div>
    </div>
  {/each}
</div>

<style>
  .traffic-light {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 3px 3px 6px;
  }

  .light-container {
    position: relative;
    width: 36px;
    height: 36px;
  }

  .light {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: var(--light-color);
    transition: background 0.3s ease, box-shadow 0.3s ease;
    box-shadow: inset 0 -2px 4px rgba(0, 0, 0, 0.3),
                inset 0 2px 4px rgba(255, 255, 255, 0.1);
  }

  .light-inner {
    position: absolute;
    inset: 4px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 30%,
                rgba(255, 255, 255, 0.4) 0%,
                transparent 70%);
    pointer-events: none;
  }

  .light-reflect {
    position: absolute;
    top: 3px;
    left: 6px;
    width: 10px;
    height: 6px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.5);
    filter: blur(1px);
    pointer-events: none;
  }

  .light-container.active .light {
    box-shadow:
      inset 0 -2px 4px rgba(0, 0, 0, 0.3),
      inset 0 2px 4px rgba(255, 255, 255, 0.1),
      0 0 12px 2px var(--light-glow),
      0 0 24px 4px var(--light-glow);
  }

  /* 闪烁动画 */
  .light-container.blink.active .light {
    animation: blink var(--blink-speed) ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  /* 呼吸动画 */
  .light-container.breathing.active .light {
    animation: breathe 2s ease-in-out infinite;
  }

  @keyframes breathe {
    0%, 100% {
      opacity: 1;
      box-shadow: inset 0 -2px 4px rgba(0,0,0,0.3), inset 0 2px 4px rgba(255,255,255,0.1), 0 0 8px var(--light-glow);
    }
    50% {
      opacity: 0.5;
      box-shadow: inset 0 -2px 4px rgba(0,0,0,0.3), inset 0 2px 4px rgba(255,255,255,0.1), 0 0 20px var(--light-glow);
    }
  }
</style>
