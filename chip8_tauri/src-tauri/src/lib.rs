// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use chip8_driver::Driver;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

type DriverState = Arc<Mutex<Option<Driver>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmulatorInfo {
    width: usize,
    height: usize,
    is_running: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameBuffer {
    data: Vec<u8>,
    updated: bool,
}

#[tauri::command]
async fn initialize_emulator(
    cpu_speed: u64,
    driver_state: State<'_, DriverState>,
) -> Result<EmulatorInfo, String> {
    let driver = Driver::new(cpu_speed).map_err(|e| format!("Failed to create driver: {}", e))?;

    let info = EmulatorInfo {
        width: chip8_driver::pixels_width(),
        height: chip8_driver::pixels_height(),
        is_running: false,
    };

    *driver_state.lock().unwrap() = Some(driver);
    Ok(info)
}

#[tauri::command]
async fn load_rom(rom_data: Vec<u8>, driver_state: State<'_, DriverState>) -> Result<(), String> {
    let mut driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_mut() {
        driver
            .reset()
            .map_err(|e| format!("Failed to reset: {}", e))?;
        driver
            .load_rom(&rom_data)
            .map_err(|e| format!("Failed to load ROM: {}", e))?;
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[tauri::command]
async fn tick_emulator(driver_state: State<'_, DriverState>) -> Result<(), String> {
    let mut driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_mut() {
        driver.tick().map_err(|e| format!("Tick failed: {}", e))?;
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[tauri::command]
async fn get_framebuffer(driver_state: State<'_, DriverState>) -> Result<FrameBuffer, String> {
    let mut driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_mut() {
        let framebuffer = driver.framebuffer().to_vec();
        let updated = driver.is_display_updated();
        if updated {
            driver.clear_display_updated_flag();
        }
        Ok(FrameBuffer {
            data: framebuffer,
            updated,
        })
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[tauri::command]
async fn key_press(key: u8, driver_state: State<'_, DriverState>) -> Result<(), String> {
    let mut driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_mut() {
        driver.key_press(key);
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[tauri::command]
async fn key_release(key: u8, driver_state: State<'_, DriverState>) -> Result<(), String> {
    let mut driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_mut() {
        driver.key_release(key);
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[tauri::command]
async fn should_beep(driver_state: State<'_, DriverState>) -> Result<bool, String> {
    let driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_ref() {
        Ok(driver.should_beep())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[tauri::command]
async fn reset_emulator(driver_state: State<'_, DriverState>) -> Result<(), String> {
    let mut driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_mut() {
        driver
            .reset()
            .map_err(|e| format!("Failed to reset: {}", e))?;
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[tauri::command]
async fn set_cpu_speed(cpu_speed: u64, driver_state: State<'_, DriverState>) -> Result<(), String> {
    let mut driver_guard = driver_state.lock().unwrap();
    if let Some(driver) = driver_guard.as_mut() {
        driver.set_cpu_speed(cpu_speed);
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let driver_state: DriverState = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(driver_state)
        .invoke_handler(tauri::generate_handler![
            initialize_emulator,
            load_rom,
            tick_emulator,
            get_framebuffer,
            key_press,
            key_release,
            should_beep,
            reset_emulator,
            set_cpu_speed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
