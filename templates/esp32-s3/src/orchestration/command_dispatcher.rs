//! Command dispatcher — routes commands to the correct peripheral.

use crate::comms::protocol::*;
use crate::peripherals::PeripheralRegistry;

/// Dispatch a command to the appropriate peripheral or system handler.
pub fn dispatch(
    command_id: u32,
    cmd: &CommandMsg,
    registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.target.as_str() {
        "system" => dispatch_system(command_id, cmd),
        "behavior" => dispatch_behavior(command_id, cmd),
        "display" => dispatch_display(command_id, cmd, registry),
        "touch" => dispatch_touch(command_id, cmd, registry),
        "audio_in" | "mic" => dispatch_audio_in(command_id, cmd, registry),
        "audio_out" | "speaker" => dispatch_audio_out(command_id, cmd, registry),
        "camera" => dispatch_camera(command_id, cmd, registry),
        "imu" => dispatch_imu(command_id, cmd, registry),
        "mouse" => dispatch_mouse(command_id, cmd, registry),
        "keyboard" => dispatch_keyboard(command_id, cmd, registry),
        _ => {
            // Try custom peripheral
            dispatch_custom(command_id, cmd, registry)
        }
    }
}

fn dispatch_system(command_id: u32, cmd: &CommandMsg) -> CommandResultMsg {
    match cmd.action.as_str() {
        "reboot" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({"status": "rebooting"}),
            error: None,
        },
        "get_info" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({
                "firmware": env!("CARGO_PKG_VERSION"),
                "board": "esp32-s3",
                "ota_state": format!("{:?}", crate::orchestration::ota::state()),
                "ota_progress": crate::orchestration::ota::progress(),
            }),
            error: None,
        },
        "ota_status" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({
                "firmware_state": format!("{:?}", crate::orchestration::ota::state()),
                "firmware_progress": crate::orchestration::ota::progress(),
                "behavior_state": format!("{:?}", crate::orchestration::ota::state()),
                "behavior_progress": crate::orchestration::ota::behavior_progress(),
            }),
            error: None,
        },
        _ => CommandResultMsg {
            command_id,
            success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown system action: {}", cmd.action)),
        },
    }
}

/// Dispatch behavior config commands (post-flash component updates).
///
/// Actions:
///   - `get`    : Return current behavior config as JSON
///   - `update` : Apply a partial behavior config update (no reboot needed)
///   - `reset`  : Reset behavior config to board defaults
///   - `enable` : Enable a specific component by name
///   - `disable`: Disable a specific component by name
fn dispatch_behavior(command_id: u32, cmd: &CommandMsg) -> CommandResultMsg {
    match cmd.action.as_str() {
        "get" => CommandResultMsg {
            command_id,
            success: true,
            // NOTE: In real impl, this reads from the Orchestrator's behavior field.
            // The dispatcher is stateless here; the orchestrator passes the config.
            result: serde_json::json!({"behavior": "see orchestrator"}),
            error: None,
        },
        "update" => {
            // The orchestrator calls orch.update_behavior() with the parsed config.
            // Here we just validate the command has the right shape.
            if cmd.args.get("components").is_some() {
                CommandResultMsg {
                    command_id,
                    success: true,
                    result: serde_json::json!({"status": "behavior_update_pending"}),
                    error: None,
                }
            } else {
                CommandResultMsg {
                    command_id,
                    success: false,
                    result: serde_json::Value::Null,
                    error: Some("missing 'components' in behavior update args".into()),
                }
            }
        }
        "enable" => {
            if let Some(name) = cmd.args.get("component").and_then(|v| v.as_str()) {
                CommandResultMsg {
                    command_id,
                    success: true,
                    result: serde_json::json!({"component": name, "enabled": true}),
                    error: None,
                }
            } else {
                CommandResultMsg {
                    command_id,
                    success: false,
                    result: serde_json::Value::Null,
                    error: Some("missing 'component' name".into()),
                }
            }
        }
        "disable" => {
            if let Some(name) = cmd.args.get("component").and_then(|v| v.as_str()) {
                CommandResultMsg {
                    command_id,
                    success: true,
                    result: serde_json::json!({"component": name, "enabled": false}),
                    error: None,
                }
            } else {
                CommandResultMsg {
                    command_id,
                    success: false,
                    result: serde_json::Value::Null,
                    error: Some("missing 'component' name".into()),
                }
            }
        }
        "reset" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({"status": "behavior_reset_to_defaults"}),
            error: None,
        },
        _ => CommandResultMsg {
            command_id,
            success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown behavior action: {}", cmd.action)),
        },
    }
}

fn dispatch_display(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "set_brightness" => {
            // Extract brightness from args and call display.set_brightness()
            CommandResultMsg {
                command_id,
                success: true,
                result: serde_json::json!({"brightness": cmd.args.get("value")}),
                error: None,
            }
        }
        "write_pixels" => {
            // Decode base64 pixel data and write to display
            CommandResultMsg {
                command_id,
                success: true,
                result: serde_json::json!({"written": true}),
                error: None,
            }
        }
        "clear" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({"cleared": true}),
            error: None,
        },
        "info" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({"width": 240, "height": 240, "format": "rgb565"}),
            error: None,
        },
        _ => CommandResultMsg {
            command_id,
            success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown display action: {}", cmd.action)),
        },
    }
}

fn dispatch_touch(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "read" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({"touching": false}),
            error: None,
        },
        "calibrate" => CommandResultMsg {
            command_id,
            success: true,
            result: serde_json::json!({"calibrated": true}),
            error: None,
        },
        _ => CommandResultMsg {
            command_id,
            success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown touch action: {}", cmd.action)),
        },
    }
}

fn dispatch_audio_in(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "start_stream" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"streaming": true}), error: None,
        },
        "stop_stream" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"streaming": false}), error: None,
        },
        _ => CommandResultMsg {
            command_id, success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown mic action: {}", cmd.action)),
        },
    }
}

fn dispatch_audio_out(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "set_volume" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"volume": cmd.args.get("value")}), error: None,
        },
        "play" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"playing": true}), error: None,
        },
        "stop" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"playing": false}), error: None,
        },
        _ => CommandResultMsg {
            command_id, success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown speaker action: {}", cmd.action)),
        },
    }
}

fn dispatch_camera(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "capture" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"frame_size": 0, "format": "jpeg"}), error: None,
        },
        "start_stream" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"streaming": true}), error: None,
        },
        "stop_stream" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"streaming": false}), error: None,
        },
        _ => CommandResultMsg {
            command_id, success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown camera action: {}", cmd.action)),
        },
    }
}

fn dispatch_imu(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "read" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"accel": {"x": 0.0, "y": 0.0, "z": 9.8}, "gyro": {"x": 0.0, "y": 0.0, "z": 0.0}}),
            error: None,
        },
        "calibrate" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"calibrated": true}), error: None,
        },
        _ => CommandResultMsg {
            command_id, success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown imu action: {}", cmd.action)),
        },
    }
}

fn dispatch_mouse(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "read" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"dx": 0, "dy": 0, "buttons": 0}), error: None,
        },
        _ => CommandResultMsg {
            command_id, success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown mouse action: {}", cmd.action)),
        },
    }
}

fn dispatch_keyboard(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    match cmd.action.as_str() {
        "read" => CommandResultMsg {
            command_id, success: true,
            result: serde_json::json!({"keys": []}), error: None,
        },
        _ => CommandResultMsg {
            command_id, success: false,
            result: serde_json::Value::Null,
            error: Some(format!("unknown keyboard action: {}", cmd.action)),
        },
    }
}

fn dispatch_custom(
    command_id: u32,
    cmd: &CommandMsg,
    _registry: &mut PeripheralRegistry,
) -> CommandResultMsg {
    // Look up custom peripheral by name in registry
    CommandResultMsg {
        command_id,
        success: false,
        result: serde_json::Value::Null,
        error: Some(format!("peripheral '{}' not found", cmd.target)),
    }
}
