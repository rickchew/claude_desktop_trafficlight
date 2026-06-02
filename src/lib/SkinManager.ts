import { writable, type Writable } from "svelte/store";
import type { Skin, SkinPayload } from "./types";
import { invoke } from "@tauri-apps/api/core";

/** 当前皮肤 store */
export const currentSkin: Writable<Skin | null> = writable(null);

/** 皮肤列表 store */
export const skinList: Writable<string[]> = writable([]);

/**
 * 加载当前皮肤
 */
export async function loadCurrentSkin(): Promise<Skin | null> {
  try {
    const payload: SkinPayload = await invoke("get_current_skin");
    const skin = payloadToSkin(payload);
    currentSkin.set(skin);
    return skin;
  } catch (e) {
    console.error("Failed to load current skin:", e);
    return null;
  }
}

/**
 * 拉取皮肤列表
 */
export async function loadSkinList(): Promise<string[]> {
  try {
    const list: string[] = await invoke("list_skins");
    skinList.set(list);
    return list;
  } catch (e) {
    console.error("Failed to load skin list:", e);
    return [];
  }
}

/**
 * 切换皮肤
 */
export async function switchSkin(name: string): Promise<Skin | null> {
  try {
    const payload: SkinPayload = await invoke("switch_skin", { name });
    const skin = payloadToSkin(payload);
    currentSkin.set(skin);
    return skin;
  } catch (e) {
    console.error(`Failed to switch skin to '${name}':`, e);
    return null;
  }
}

/** 将后端 payload 转为 Skin */
function payloadToSkin(p: SkinPayload): Skin {
  return {
    name: p.name,
    description: p.description,
    lights: p.lights,
    background: p.background,
    border: p.border,
    label: p.label,
  };
}
