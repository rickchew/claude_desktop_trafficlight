use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 皮肤定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    pub name: String,
    pub description: String,
    pub lights: LightColors,
    pub background: BackgroundConfig,
    pub border: BorderConfig,
    pub label: TextStyle,
}

/// 灯色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightColors {
    pub red: LightConfig,
    pub yellow: LightConfig,
    pub green: LightConfig,
    pub gray: LightConfig,
}

/// 单个灯配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightConfig {
    pub on: String,   // 亮色 (CSS color)
    pub off: String,  // 灭色
    pub glow: Option<String>, // 辉光效果色
}

/// 背景配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    pub color: String,
    pub opacity: f64,
    pub blur: Option<bool>,
}

/// 边框配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderConfig {
    pub color: String,
    pub radius: String,
    pub width: String,
}

/// 文字样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub color: String,
    pub size: String,
    pub font_family: Option<String>,
}

/// 皮肤管理器
pub struct SkinManager {
    skins_dir: PathBuf,
    skins: HashMap<String, Skin>,
    current_skin: String,
}

impl SkinManager {
    /// 创建皮肤管理器，从指定目录加载皮肤
    pub fn new(skins_dir: PathBuf) -> Self {
        let mut manager = Self {
            skins_dir,
            skins: HashMap::new(),
            current_skin: "default".to_string(),
        };
        manager.load_all();
        manager
    }

    /// 加载所有皮肤
    fn load_all(&mut self) {
        if !self.skins_dir.exists() {
            // 内建默认皮肤
            self.load_builtin();
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.skins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let theme_file = path.join("theme.json");
                    if theme_file.exists() {
                        if let Ok(content) = fs::read_to_string(&theme_file) {
                            if let Ok(skin) = serde_json::from_str::<Skin>(&content) {
                                let name = skin.name.clone();
                                self.skins.insert(name, skin);
                            }
                        }
                    }
                }
            }
        }

        // 如果没有任何皮肤加载，使用内建默认
        if self.skins.is_empty() {
            self.load_builtin();
        }
    }

    /// 加载内建默认皮肤
    fn load_builtin(&mut self) {
        let default = Skin {
            name: "default".to_string(),
            description: "经典红绿灯风格".to_string(),
            lights: LightColors {
                red: LightConfig {
                    on: "#FF3B30".to_string(),
                    off: "#4A1A18".to_string(),
                    glow: Some("rgba(255, 59, 48, 0.6)".to_string()),
                },
                yellow: LightConfig {
                    on: "#FFD60A".to_string(),
                    off: "#4A3A0A".to_string(),
                    glow: Some("rgba(255, 214, 10, 0.6)".to_string()),
                },
                green: LightConfig {
                    on: "#30D158".to_string(),
                    off: "#1A3A22".to_string(),
                    glow: Some("rgba(48, 209, 88, 0.6)".to_string()),
                },
                gray: LightConfig {
                    on: "#8E8E93".to_string(),
                    off: "#3A3A3C".to_string(),
                    glow: None,
                },
            },
            background: BackgroundConfig {
                color: "#1C1C1E".to_string(),
                opacity: 0.85,
                blur: Some(true),
            },
            border: BorderConfig {
                color: "#3A3A3C".to_string(),
                radius: "16px".to_string(),
                width: "1px".to_string(),
            },
            label: TextStyle {
                color: "#EBEBF5".to_string(),
                size: "11px".to_string(),
                font_family: Some("system-ui, -apple-system, sans-serif".to_string()),
            },
        };
        self.skins.insert("default".to_string(), default);
    }

    /// 获取当前皮肤
    pub fn current(&self) -> Option<&Skin> {
        self.skins.get(&self.current_skin)
    }

    /// 切换皮肤
    pub fn switch(&mut self, name: &str) -> Option<&Skin> {
        if self.skins.contains_key(name) {
            self.current_skin = name.to_string();
            return self.current();
        }
        None
    }

    /// 获取所有皮肤名称
    pub fn list(&self) -> Vec<&str> {
        self.skins.keys().map(|s| s.as_str()).collect()
    }

    /// 获取当前皮肤名称
    pub fn current_name(&self) -> &str {
        &self.current_skin
    }

    /// 获取皮肤列表（含描述）
    pub fn list_with_desc(&self) -> Vec<(&str, &str)> {
        self.skins
            .iter()
            .map(|(name, skin)| (name.as_str(), skin.description.as_str()))
            .collect()
    }
}
