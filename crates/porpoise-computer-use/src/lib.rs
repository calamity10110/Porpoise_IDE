use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use porpoise_core::error::{PorpoiseError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClickButton {
    Left,
    Right,
    Middle,
}

pub struct ComputerUse {
    enigo: Enigo,
}

impl ComputerUse {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| PorpoiseError::Runtime(format!("enigo init: {e}")))?;
        Ok(Self { enigo })
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| PorpoiseError::Runtime(format!("mouse move: {e}")))?;
        Ok(())
    }

    pub fn mouse_click(&mut self, button: ClickButton) -> Result<()> {
        let btn = match button {
            ClickButton::Left => Button::Left,
            ClickButton::Right => Button::Right,
            ClickButton::Middle => Button::Middle,
        };
        self.enigo
            .button(btn, Direction::Click)
            .map_err(|e| PorpoiseError::Runtime(format!("mouse click: {e}")))?;
        Ok(())
    }

    pub fn type_text(&mut self, text: &str) -> Result<()> {
        self.enigo
            .text(text)
            .map_err(|e| PorpoiseError::Runtime(format!("type: {e}")))?;
        Ok(())
    }

    pub fn key_press(&mut self, key: &str) -> Result<()> {
        let k = match key.to_lowercase().as_str() {
            "enter" => Key::Return,
            "tab" => Key::Tab,
            "space" => Key::Space,
            "escape" | "esc" => Key::Escape,
            "backspace" => Key::Backspace,
            "delete" => Key::Delete,
            "up" => Key::UpArrow,
            "down" => Key::DownArrow,
            "left" => Key::LeftArrow,
            "right" => Key::RightArrow,
            "home" => Key::Home,
            "end" => Key::End,
            _ => return Err(PorpoiseError::Runtime(format!("unknown key: {key}"))),
        };
        self.enigo
            .key(k, Direction::Click)
            .map_err(|e| PorpoiseError::Runtime(format!("key press: {e}")))?;
        Ok(())
    }

    pub async fn execute_action(&mut self, action: &AutomationAction) -> Result<serde_json::Value> {
        match action {
            AutomationAction::MouseMove { x, y } => {
                self.mouse_move(*x, *y)?;
                Ok(serde_json::json!({"action": "mouse_move", "x": x, "y": y}))
            }
            AutomationAction::MouseClick { button } => {
                self.mouse_click(button.clone())?;
                Ok(serde_json::json!({"action": "mouse_click", "button": format!("{button:?}")}))
            }
            AutomationAction::TypeText { text } => {
                self.type_text(text)?;
                Ok(serde_json::json!({"action": "type_text", "length": text.len()}))
            }
            AutomationAction::KeyPress { key } => {
                self.key_press(key)?;
                Ok(serde_json::json!({"action": "key_press", "key": key}))
            }
            AutomationAction::Screenshot => Err(PorpoiseError::Unimplemented("screenshot")),
            AutomationAction::Wait { duration_ms } => {
                tokio::time::sleep(std::time::Duration::from_millis(*duration_ms)).await;
                Ok(serde_json::json!({"action": "wait", "ms": duration_ms}))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationAction {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: ClickButton },
    TypeText { text: String },
    KeyPress { key: String },
    Screenshot,
    Wait { duration_ms: u64 },
}
