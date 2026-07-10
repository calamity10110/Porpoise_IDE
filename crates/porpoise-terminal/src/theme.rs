use std::{collections::HashMap, path::Path};

use porpoise_core::error::{PorpoiseError, Result};

use crate::types::ColorScheme;

/// Imports a color scheme from an Alacritty YAML config file.
///
/// Alacritty uses a `colors:` section with `primary:` (foreground/background),
/// `normal:` and `bright:` color groups. Hex strings like `0x1a1b26` are
/// parsed to `#RRGGBB` format.
pub fn import_alacritty(path: &Path) -> Result<ColorScheme> {
    let content =
        std::fs::read_to_string(path).map_err(|e| PorpoiseError::Terminal(format!("read alacritty config: {e}")))?;

    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| PorpoiseError::Terminal(format!("parse alacritty YAML: {e}")))?;

    let colors = yaml
        .get("colors")
        .ok_or_else(|| PorpoiseError::Terminal("no 'colors' section in alacritty config".into()))?;

    let primary = colors.get("primary").cloned().unwrap_or_default();
    let normal = colors.get("normal").cloned().unwrap_or_default();
    let bright = colors.get("bright").cloned().unwrap_or_default();

    let mut scheme = ColorScheme::default();

    if let Some(fg) = yaml_str(&primary, "foreground") {
        scheme.foreground = fg;
    }
    if let Some(bg) = yaml_str(&primary, "background") {
        scheme.background = bg;
    }

    scheme.black = yaml_str(&normal, "black").unwrap_or_default();
    scheme.red = yaml_str(&normal, "red").unwrap_or_default();
    scheme.green = yaml_str(&normal, "green").unwrap_or_default();
    scheme.yellow = yaml_str(&normal, "yellow").unwrap_or_default();
    scheme.blue = yaml_str(&normal, "blue").unwrap_or_default();
    scheme.magenta = yaml_str(&normal, "magenta").unwrap_or_default();
    scheme.cyan = yaml_str(&normal, "cyan").unwrap_or_default();
    scheme.white = yaml_str(&normal, "white").unwrap_or_default();

    scheme.bright_black = yaml_str(&bright, "black").unwrap_or_default();
    scheme.bright_red = yaml_str(&bright, "red").unwrap_or_default();
    scheme.bright_green = yaml_str(&bright, "green").unwrap_or_default();
    scheme.bright_yellow = yaml_str(&bright, "yellow").unwrap_or_default();
    scheme.bright_blue = yaml_str(&bright, "blue").unwrap_or_default();
    scheme.bright_magenta = yaml_str(&bright, "magenta").unwrap_or_default();
    scheme.bright_cyan = yaml_str(&bright, "cyan").unwrap_or_default();
    scheme.bright_white = yaml_str(&bright, "white").unwrap_or_default();

    Ok(scheme)
}

/// Imports a color scheme from an iTerm2 plist file.
///
/// iTerm2 stores color schemes as XML plist under
/// `Background Color`, `Foreground Color`, `Ansi 0 Color` through `Ansi 15 Color`.
/// Each color has `Red Component`, `Green Component`, `Blue Component` as
/// float values 0.0–1.0.
pub fn import_iterm2(path: &Path) -> Result<ColorScheme> {
    let content =
        std::fs::read_to_string(path).map_err(|e| PorpoiseError::Terminal(format!("read iterm2 plist: {e}")))?;

    let plist: HashMap<String, plist::Value> = plist::from_bytes(content.as_bytes())
        .map_err(|e| PorpoiseError::Terminal(format!("parse iterm2 plist: {e}")))?;

    let scheme = ColorScheme {
        background: get_iterm2_color(&plist, "Background Color"),
        foreground: get_iterm2_color(&plist, "Foreground Color"),
        black: get_iterm2_color(&plist, "Ansi 0 Color"),
        red: get_iterm2_color(&plist, "Ansi 1 Color"),
        green: get_iterm2_color(&plist, "Ansi 2 Color"),
        yellow: get_iterm2_color(&plist, "Ansi 3 Color"),
        blue: get_iterm2_color(&plist, "Ansi 4 Color"),
        magenta: get_iterm2_color(&plist, "Ansi 5 Color"),
        cyan: get_iterm2_color(&plist, "Ansi 6 Color"),
        white: get_iterm2_color(&plist, "Ansi 7 Color"),
        bright_black: get_iterm2_color(&plist, "Ansi 8 Color"),
        bright_red: get_iterm2_color(&plist, "Ansi 9 Color"),
        bright_green: get_iterm2_color(&plist, "Ansi 10 Color"),
        bright_yellow: get_iterm2_color(&plist, "Ansi 11 Color"),
        bright_blue: get_iterm2_color(&plist, "Ansi 12 Color"),
        bright_magenta: get_iterm2_color(&plist, "Ansi 13 Color"),
        bright_cyan: get_iterm2_color(&plist, "Ansi 14 Color"),
        bright_white: get_iterm2_color(&plist, "Ansi 15 Color"),
    };

    Ok(scheme)
}

fn yaml_str(map: &serde_yaml::Value, key: &str) -> Option<String> {
    map.get(key).and_then(|v| v.as_str()).map(normalize_hex)
}

fn normalize_hex(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("0x") {
        format!("#{rest}")
    } else if trimmed.starts_with('#') {
        trimmed.to_string()
    } else {
        format!("#{trimmed}")
    }
}

fn get_iterm2_color(plist: &HashMap<String, plist::Value>, key: &str) -> String {
    let entry = plist.get(key).and_then(|v| v.as_dictionary());
    if let Some(dict) = entry {
        let r = dict.get("Red Component").and_then(|v| v.as_real()).unwrap_or(0.0);
        let g = dict.get("Green Component").and_then(|v| v.as_real()).unwrap_or(0.0);
        let b = dict.get("Blue Component").and_then(|v| v.as_real()).unwrap_or(0.0);
        format!(
            "#{:02x}{:02x}{:02x}",
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8
        )
    } else {
        String::new()
    }
}

/// Returns built-in color schemes.
pub fn builtin_schemes() -> Vec<(&'static str, ColorScheme)> {
    vec![
        (
            "tokyo-night",
            ColorScheme {
                foreground: "#c0caf5".into(),
                background: "#1a1b26".into(),
                black: "#15161e".into(),
                red: "#f7768e".into(),
                green: "#9ece6a".into(),
                yellow: "#e0af68".into(),
                blue: "#7aa2f7".into(),
                magenta: "#bb9af7".into(),
                cyan: "#7dcfff".into(),
                white: "#a9b1d6".into(),
                bright_black: "#414868".into(),
                bright_red: "#f7768e".into(),
                bright_green: "#9ece6a".into(),
                bright_yellow: "#e0af68".into(),
                bright_blue: "#7aa2f7".into(),
                bright_magenta: "#bb9af7".into(),
                bright_cyan: "#7dcfff".into(),
                bright_white: "#c0caf5".into(),
            },
        ),
        (
            "dracula",
            ColorScheme {
                foreground: "#f8f8f2".into(),
                background: "#282a36".into(),
                black: "#21222c".into(),
                red: "#ff5555".into(),
                green: "#50fa7b".into(),
                yellow: "#f1fa8c".into(),
                blue: "#bd93f9".into(),
                magenta: "#ff79c6".into(),
                cyan: "#8be9fd".into(),
                white: "#f8f8f2".into(),
                bright_black: "#6272a4".into(),
                bright_red: "#ff6e67".into(),
                bright_green: "#69ff94".into(),
                bright_yellow: "#fffa50".into(),
                bright_blue: "#d6acff".into(),
                bright_magenta: "#ff92df".into(),
                bright_cyan: "#a4ffff".into(),
                bright_white: "#ffffff".into(),
            },
        ),
        (
            "solarized-dark",
            ColorScheme {
                foreground: "#93a1a1".into(),
                background: "#002b36".into(),
                black: "#073642".into(),
                red: "#dc322f".into(),
                green: "#859900".into(),
                yellow: "#b58900".into(),
                blue: "#268bd2".into(),
                magenta: "#d33682".into(),
                cyan: "#2aa198".into(),
                white: "#eee8d5".into(),
                bright_black: "#002b36".into(),
                bright_red: "#cb4b16".into(),
                bright_green: "#586e75".into(),
                bright_yellow: "#657b83".into(),
                bright_blue: "#839496".into(),
                bright_magenta: "#6c71c4".into(),
                bright_cyan: "#93a1a1".into(),
                bright_white: "#fdf6e3".into(),
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hex() {
        assert_eq!(normalize_hex("0x1a1b26"), "#1a1b26");
        assert_eq!(normalize_hex("#ff0000"), "#ff0000");
        assert_eq!(normalize_hex("abcdef"), "#abcdef");
    }

    #[test]
    fn test_builtin_schemes() {
        let schemes = builtin_schemes();
        assert!(schemes.len() >= 3);
        assert_eq!(schemes[0].0, "tokyo-night");
        assert!(!schemes[0].1.foreground.is_empty());
    }
}
