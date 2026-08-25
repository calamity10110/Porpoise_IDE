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
    let wifi_config = WifiConfig::default();
    let mut wifi = WifiManager::new(wifi_config);
    wifi.start().ok();
    println!("WiFi: {:?}", wifi.state());

    // --- Step 5: Servers ---
    let http = HttpServer::new(80);
    let mut ws = WebSocketServer::new(81);
    ws.listen().ok();
    println!("HTTP :80, WebSocket :81");

    // --- Step 6: Orchestrator ---
    let mut orch = Orchestrator::new();
    orch.set_running();
    println!("System running. Entering main loop.");

    // --- Step 7: Main loop ---
    loop {
        // Poll peripherals for new data
        // registry.poll_all();

        // Process any pending WebSocket commands
        // while let Some(cmd) = ws.recv_command() {
        //     let result = orch.dispatch_command(&cmd, &mut registry);
        //     ws.send_response(&result).ok();
        // }

        // Send periodic telemetry (every ~1s)
        // let telemetry = orch.build_telemetry(&registry);
        // ws.broadcast_telemetry(&telemetry).ok();

        // Yield to other embassy tasks
        // embassy_time::Timer::after(Duration::from_millis(10)).await;
    }
}
