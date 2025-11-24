use brightness::{Brightness, BrightnessDevice};
use chrono::{Local, Timelike};
use futures::TryStreamExt;
use std::{thread, time::Duration};

fn main() {
    loop {
        let now = Local::now().time();
        let hour = now.hour();
        let minute = now.minute();

        let target_brightness = calculate_target(hour, minute);

        // Apply brightness with tiny increments to keep changes smooth
        if let Err(e) = apply_smooth_brightness(target_brightness) {
            eprintln!("Failed to adjust brightness: {}", e);
        }

        thread::sleep(Duration::from_secs(30));
    }
}

/// Brightness schedule:
/// 100% normally
/// 19:00 → start fading
/// 22:00 → reach 10%
fn calculate_target(hour: u32, minute: u32) -> u32 {
    let minutes = hour * 60 + minute;

    // Evening fade: 19:00 → 22:00
    let evening_start = 19 * 60;
    let evening_end = 22 * 60;

    // Morning recovery: 00:00 → 10:00
    let morning_start = 0;
    let morning_end = 10 * 60;

    if minutes >= evening_start && minutes < evening_end {
        let progress = (minutes - evening_start) as f32 / (evening_end - evening_start) as f32;
        return (100.0 - progress * 90.0).round() as u32;
    }

    if minutes >= morning_start && minutes < morning_end {
        let progress = (minutes - morning_start) as f32 / (morning_end - morning_start) as f32;
        return (progress * 90.0).round() as u32;
    }

    // Normal daytime 07:00 → 19:00 → 100%
    if minutes >= 10 * 60 && minutes < 19 * 60 {
        return 100;
    }

    // Night 22:00 → 00:00 → hold 10%
    if minutes >= 22 * 60 {
        return 0;
    }

    100
}


/// Smoothly apply brightness in steps of 1–2%
fn apply_smooth_brightness(target: u32) -> Result<(), brightness::Error> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let devices = brightness::brightness_devices();

        devices
            .try_for_each(|mut dev| async move {
                let current = dev.get().await?;

                if current == target {
                    return Ok(());
                }

                let step = if current > target { -1 } else { 1 };

                let mut level = current as i32;

                while level != target as i32 {
                    level += step;
                    dev.set(level as u32).await?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }

                Ok(())
            })
            .await?;

        Ok(())
    })
}
