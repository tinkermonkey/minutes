use cpal::traits::{DeviceTrait, HostTrait};
use crate::{db, state::AppState};

#[derive(Debug, serde::Serialize)]
pub struct AudioDevice {
    pub name:       String,
    pub is_default: bool,
}

/// List all available audio input devices, flagging the system default.
#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let default_name = host.default_input_device()
        .and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| e.to_string())?
        .filter_map(|d| {
            d.name().ok().map(|name| {
                let is_default = default_name.as_deref() == Some(name.as_str());
                AudioDevice { name, is_default }
            })
        })
        .collect();

    Ok(devices)
}

/// Persist the user's preferred audio input device and update live state.
#[tauri::command]
pub fn set_audio_device(
    device_name: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().expect("db mutex poisoned");
    db::settings::set(&db, "preferred_audio_device", &device_name)
        .map_err(|e| e.to_string())?;
    drop(db);
    *state.preferred_device.lock().expect("preferred_device mutex poisoned") = Some(device_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_audio_devices_returns_ok() {
        // Smoke test: verifies the CPAL host enumeration path does not panic.
        // On CI without audio hardware this returns an empty vec, which is fine.
        let result = get_audio_devices();
        assert!(result.is_ok(), "get_audio_devices should not error: {:?}", result);
    }
}
