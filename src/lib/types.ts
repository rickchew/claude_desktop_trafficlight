/** 红绿灯状态 */
export type LightState =
  | "starting"
  | "working"
  | "thinking"
  | "attention"
  | "error"
  | "idle"
  | "done"
  | "stopped";

/** 灯色分组 */
export type ColorGroup = "red" | "yellow" | "green" | "gray";

/** 动画类型 */
export type AnimationType = "solid" | "slow-blink" | "fast-blink" | "breathing" | "off";

/** 状态变化事件载荷 */
export interface StatePayload {
  state: LightState;
  colorGroup: ColorGroup;
  animation: AnimationType;
  blinkInterval: number;
  label: string;
  timestamp: string;
}

/** 单个灯配置 */
export interface LightConfig {
  on: string;
  off: string;
  glow?: string;
}

/** 灯色集合 */
export interface LightColors {
  red: LightConfig;
  yellow: LightConfig;
  green: LightConfig;
  gray: LightConfig;
}

/** 背景配置 */
export interface BackgroundConfig {
  color: string;
  opacity: number;
  blur?: boolean;
}

/** 边框配置 */
export interface BorderConfig {
  color: string;
  radius: string;
  width: string;
}

/** 文字样式 */
export interface TextStyle {
  color: string;
  size: string;
  fontFamily?: string;
}

/** 皮肤主题 */
export interface Skin {
  name: string;
  description: string;
  lights: LightColors;
  background: BackgroundConfig;
  border: BorderConfig;
  label: TextStyle;
}

/** 皮肤变化事件载荷 */
export interface SkinPayload {
  name: string;
  description: string;
  lights: LightColors;
  background: BackgroundConfig;
  border: BorderConfig;
  label: TextStyle;
}

/** 状态中文映射 */
export const STATE_LABELS: Record<LightState, string> = {
  starting: "启动中",
  working: "工作中",
  thinking: "思考中",
  attention: "需要交互",
  error: "错误",
  idle: "空闲",
  done: "完成",
  stopped: "已停止",
};
