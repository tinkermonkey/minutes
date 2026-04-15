use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::sync::mpsc;

/// Holds the live capture stream and the sample receiver.
///
/// Dropping this struct stops capture — `_stream` must remain alive for audio
/// to flow. `rx` drains the samples produced by the CPAL callback.
pub struct CaptureHandle {
    pub rx:      mpsc::Receiver<Vec<f32>>,
    /// Kept alive here; dropping it stops the stream.
    pub _stream: cpal::Stream,
}

/// Open the default (or named) input device at 16 kHz mono f32 and return a
/// [`CaptureHandle`] whose `rx` yields batches of samples as they arrive.
///
/// The CPAL callback runs on an OS audio thread. Samples are sent via a
/// bounded `mpsc` channel; if the consumer falls behind, frames are dropped
/// rather than blocking the audio thread.
pub fn start_capture(preferred: Option<&str>) -> anyhow::Result<CaptureHandle> {
    let host = cpal::default_host();

    #[allow(deprecated)]
    let device = if let Some(name) = preferred {
        host.input_devices()?
            .find(|d| d.name().ok().as_deref() == Some(name))
            .unwrap_or_else(|| {
                host.default_input_device()
                    .expect("no default input device found")
            })
    } else {
        host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("no input device found"))?
    };

    // In cpal 0.17, SampleRate is a plain u32 type alias.
    // Prefer 16 kHz mono; fall back to the device's default config.
    let supported_config = device
        .supported_input_configs()?
        .filter(|c| c.channels() == 1)
        .find(|c| c.min_sample_rate() <= 16_000 && c.max_sample_rate() >= 16_000)
        .and_then(|c| c.try_with_sample_rate(16_000))
        .or_else(|| device.default_input_config().ok())
        .ok_or_else(|| anyhow::anyhow!("no compatible input config found on device"))?;

    let config = supported_config.config();
    let sample_rate = config.sample_rate;
    let channels = config.channels as usize;

    // Bound of 8 keeps latency low; frames are ~10 ms so 8 gives ~80 ms slack.
    let (tx, rx) = mpsc::channel::<Vec<f32>>(8);

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            // Downmix multichannel to mono.
            let mono: Vec<f32> = if channels == 1 {
                data.to_vec()
            } else {
                data.chunks(channels)
                    .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                    .collect()
            };

            // Resample to 16 kHz when the device runs at a different rate.
            let resampled = if sample_rate != 16_000 {
                resample(&mono, sample_rate, 16_000)
            } else {
                mono
            };

            // try_send: drop the batch rather than blocking the audio thread.
            let _ = tx.try_send(resampled);
        },
        |err| eprintln!("CPAL capture error: {err}"),
        None,
    )?;

    stream.play()?;

    Ok(CaptureHandle { rx, _stream: stream })
}

/// Linear interpolation resample — good enough for the 44.1 kHz → 16 kHz
/// conversion that most macOS devices need. Silero VAD operates on 16 kHz
/// input and does not perform its own downsampling.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = to_rate as f64 / from_rate as f64;
    let out_len = (samples.len() as f64 * ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_idx = i as f64 / ratio;
        let lo = src_idx.floor() as usize;
        let hi = (lo + 1).min(samples.len() - 1);
        let frac = (src_idx - lo as f64) as f32;
        out.push(samples[lo] * (1.0 - frac) + samples[hi] * frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::resample;

    #[test]
    fn resample_identity() {
        let input = vec![0.1, 0.2, 0.3, 0.4];
        assert_eq!(resample(&input, 16_000, 16_000), input);
    }

    #[test]
    fn resample_empty() {
        let result: Vec<f32> = resample(&[], 44_100, 16_000);
        assert!(result.is_empty());
    }

    #[test]
    fn resample_downsamples_length() {
        // 441 samples at 44.1 kHz → ~160 samples at 16 kHz
        let input: Vec<f32> = (0..441).map(|i| i as f32 / 441.0).collect();
        let out = resample(&input, 44_100, 16_000);
        let expected_len = (441f64 * 16_000f64 / 44_100f64).ceil() as usize;
        assert_eq!(out.len(), expected_len);
    }
}
