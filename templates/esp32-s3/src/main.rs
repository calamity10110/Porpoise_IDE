//! ESP32-S3 Porpoise Template — main entry point.
//!
//! This is the boot sequence:
//! 1. Init heap allocator (PSRAM + internal)
//! 2. Init embassy runtime
//! 3. Read board config (TOML or compile-time)
//! 4. Init peripherals based on board config
//! 5. Start WiFi (STA or AP)
//! 6. Start HTTP + WebSocket servers
//! 7. Enter main loop: poll peripherals, process commands, send telemetry

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl};
use esp_println::println;

mod board;
mod comms;
mod orchestration;
mod peripherals;
mod sensors;

use board::waveshare_lcd_349::WaveshareLcd349;
use comms::wifi::{WifiConfig, WifiManager};
use comms::http_server::HttpServer;
use comms::websocket::WebSocketServer;
use comms::security::{AuthToken, RateLimiter};
use orchestration::Orchestrator;
use peripherals::PeripheralRegistry;

#[entry]
fn main() -> ! {
    // --- Step 1: Hardware init ---
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    println!("=== Porpoise ESP32-S3 Template ===");
    println!("Board: Waveshare LCD 349");
    println!("CPU: {} MHz", 240);

    // --- Step 2: Board config ---
    let board = WaveshareLcd349;
    println!("Display: {}x{}", board.display_width(), board.display_height());

    // --- Step 3: Init peripherals ---
    let mut registry = PeripheralRegistry::new();
    // TODO: Instantiate concrete peripherals based on board config
    // registry.register_display(...);
    // registry.register_touch(...);
    // registry.register_imu(...);
    println!("Peripherals registered: {}", registry.count());

    // --- Step 4: WiFi ---
    let wifi_config = WifiConfig::ap("Porpoise-Setup", "change-me-1234");
    let mut wifi = WifiManager::new(wifi_config);
    wifi.start().ok();
    println!("WiFi: {:?}", wifi.state());

    // --- Step 5: Security ---
    // Generate or load auth token from NVS
    let auth_token = AuthToken::generate();
    let mut rate_limiter = RateLimiter::new(5, 10_000, 60_000);
    println!("Auth: enabled (token on display/UART)");
    // TODO: Display token on screen or print to UART for user to copy
    // display.show_auth_token(&auth_token);

    // --- Step 6: Servers (auth-enabled) ---
    let http = HttpServer::new(80, true);
    let mut ws = WebSocketServer::new(81, true);
    ws.listen().ok();
    println!("HTTP :80 (auth), WebSocket :81 (auth)");

    // --- Step 6: Orchestrator (with behavior config from flash or board defaults) ---
    let mut orch = Orchestrator::with_board_defaults("waveshare-349");
    orch.set_running();
    println!("System running. Behavior config: {} components", orch.behavior.components.len());
    for comp in &orch.behavior.components {
        println!("  [{}] {}", if comp.enabled { "ON" } else { "OFF" }, comp.name);
    }
    println!("Entering main loop.");

    // --- Step 7: Main loop ---
    loop {
        // Poll peripherals for new data
        // registry.poll_all();

        // Process any pending WebSocket commands
        // while let Some(cmd) = ws.recv_command() {
        //     match cmd.target.as_str() {
        //         "behavior" => {
        //             // Handle behavior config updates (no reboot needed)
        //             if cmd.action == "update" {
        //                 if let Err(e) = orch.update_behavior(&parsed_update) {
        //                     println!("Behavior update error: {:?}", e);
        //                 }
        //             }
        //         }
        //         _ => {
        //             let result = orch.dispatch_command(&cmd, &mut registry);
        //             ws.send_response(&result).ok();
        //         }
        //     }
        // }

        // Send periodic telemetry (every ~1s)
        // let telemetry = orch.build_telemetry(&registry);
        // ws.broadcast_telemetry(&telemetry).ok();

        // Yield to other embassy tasks
        // embassy_time::Timer::after(Duration::from_millis(10)).await;
    }
}
