use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct AudioRecorder {
    stop_flag: Arc<AtomicBool>,
    done_flag: Arc<AtomicBool>,
    error: Arc<Mutex<Option<String>>>,
    output_path: PathBuf,
    /// Peak RMS observed during recording (0.0–1.0). Used for VAD silence detection.
    peak_rms: Arc<Mutex<f64>>,
}

/// Minimum peak RMS to consider a recording as containing speech.
/// Below this threshold, the recording is treated as silence and skipped.
/// Calibrated for 48kHz mic input; typical speech produces 0.01–0.1 RMS.
pub const SILENCE_THRESHOLD: f64 = 0.005;

unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn start(output_path: PathBuf, app_handle: AppHandle) -> Result<Self, String> {
        let _ = std::fs::remove_file(&output_path);

        let host = cpal::default_host();
        let _device = host
            .default_input_device()
            .ok_or_else(|| "No microphone detected".to_string())?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::new(AtomicBool::new(false));
        let error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let started: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let peak_rms: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));

        let stop_clone = stop_flag.clone();
        let done_clone = done_flag.clone();
        let error_clone = error.clone();
        let started_clone = started.clone();
        let path_clone = output_path.clone();
        let peak_rms_clone = peak_rms.clone();

        std::thread::spawn(move || {
            let result = run_recording(path_clone, app_handle, stop_clone.clone(), started_clone, peak_rms_clone);
            if let Err(e) = result {
                *error_clone.lock().unwrap() = Some(e);
            }
            done_clone.store(true, Ordering::Release);
        });

        let start_time = std::time::Instant::now();
        loop {
            if started.load(Ordering::Acquire) {
                break;
            }
            if done_flag.load(Ordering::Acquire) {
                let err = error.lock().unwrap().take();
                return Err(err.unwrap_or_else(|| "Recording failed to start".to_string()));
            }
            if start_time.elapsed().as_secs() > 2 {
                stop_flag.store(true, Ordering::Release);
                return Err("Recording startup timed out".to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Ok(Self {
            stop_flag,
            done_flag,
            error,
            output_path,
            peak_rms,
        })
    }

    /// Stop recording and return (audio_path, peak_rms).
    /// peak_rms can be checked against SILENCE_THRESHOLD to detect empty recordings.
    pub fn stop(self) -> Result<(PathBuf, f64), String> {
        self.stop_flag.store(true, Ordering::Release);

        let start_time = std::time::Instant::now();
        while !self.done_flag.load(Ordering::Acquire) {
            if start_time.elapsed().as_secs() > 3 {
                return Err("Recording stop timed out".to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        if let Some(err) = self.error.lock().unwrap().take() {
            return Err(err);
        }

        if !self.output_path.exists() {
            return Err("Recording file was not created".to_string());
        }

        let metadata = std::fs::metadata(&self.output_path)
            .map_err(|e| format!("Cannot read recording file: {}", e))?;

        if metadata.len() < 1000 {
            return Err("Recording too short or empty".to_string());
        }

        let peak = *self.peak_rms.lock().unwrap();
        Ok((self.output_path.clone(), peak))
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
    }
}

fn run_recording(
    output_path: PathBuf,
    app_handle: AppHandle,
    stop_flag: Arc<AtomicBool>,
    started_flag: Arc<AtomicBool>,
    peak_rms: Arc<Mutex<f64>>,
) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No microphone detected".to_string())?;

    let default_config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get default input config: {}", e))?;

    let sample_rate = default_config.sample_rate().0;
    let device_channels = default_config.channels();
    let sample_format = default_config.sample_format();

    // Log device config for debugging
    eprintln!(
        "Audio: device='{}', rate={}Hz, channels={}, format={:?}",
        device.name().unwrap_or_else(|_| "unknown".to_string()),
        sample_rate,
        device_channels,
        sample_format
    );

    let config = cpal::StreamConfig {
        channels: device_channels,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Always write MONO WAV — Whisper works best with mono audio
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = hound::WavWriter::create(&output_path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;
    let writer = Arc::new(Mutex::new(Some(writer)));
    let writer_for_stream = writer.clone();

    let stop_for_stream = stop_flag.clone();
    let ch = device_channels as usize;

    // RMS at ~20Hz
    let emit_interval = (sample_rate / 20) as u32;
    let mut rms_accumulator: f64 = 0.0;
    let mut rms_count: u32 = 0;

    let stream_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let stream_error_clone = stream_error.clone();

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if stop_for_stream.load(Ordering::Relaxed) {
                    return;
                }
                if let Ok(mut guard) = writer_for_stream.lock() {
                    if let Some(ref mut w) = *guard {
                        // Mix down to mono: average all channels per frame
                        for frame in data.chunks(ch) {
                            let mono: f32 = frame.iter().sum::<f32>() / ch as f32;
                            let s16 = (mono.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                            let _ = w.write_sample(s16);

                            // RMS on mono signal
                            rms_accumulator += (mono as f64) * (mono as f64);
                            rms_count += 1;
                            if rms_count >= emit_interval {
                                let rms = (rms_accumulator / rms_count as f64).sqrt();
                                let _ = app_handle.emit("audio-level", rms.min(1.0));
                                // Track peak RMS for VAD silence detection
                                if let Ok(mut peak) = peak_rms.lock() {
                                    if rms > *peak {
                                        *peak = rms;
                                    }
                                }
                                rms_accumulator = 0.0;
                                rms_count = 0;
                            }
                        }
                    }
                }
            },
            move |err| {
                *stream_error_clone.lock().unwrap() =
                    Some(format!("Audio stream error: {}", err));
            },
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if stop_for_stream.load(Ordering::Relaxed) {
                    return;
                }
                if let Ok(mut guard) = writer_for_stream.lock() {
                    if let Some(ref mut w) = *guard {
                        for frame in data.chunks(ch) {
                            let mono: i32 = frame.iter().map(|&s| s as i32).sum::<i32>() / ch as i32;
                            let _ = w.write_sample(mono as i16);

                            let normalized = mono as f64 / i16::MAX as f64;
                            rms_accumulator += normalized * normalized;
                            rms_count += 1;
                            if rms_count >= emit_interval {
                                let rms = (rms_accumulator / rms_count as f64).sqrt();
                                let _ = app_handle.emit("audio-level", rms.min(1.0));
                                // Track peak RMS for VAD silence detection
                                if let Ok(mut peak) = peak_rms.lock() {
                                    if rms > *peak {
                                        *peak = rms;
                                    }
                                }
                                rms_accumulator = 0.0;
                                rms_count = 0;
                            }
                        }
                    }
                }
            },
            move |err| {
                *stream_error_clone.lock().unwrap() =
                    Some(format!("Audio stream error: {}", err));
            },
            None,
        ),
        _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
    }
    .map_err(|e| format!("Failed to build audio stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start audio stream: {}", e))?;

    started_flag.store(true, Ordering::Release);

    while !stop_flag.load(Ordering::Acquire) {
        std::thread::sleep(std::time::Duration::from_millis(20));
        if let Some(err) = stream_error.lock().unwrap().take() {
            if let Ok(mut guard) = writer.lock() {
                if let Some(w) = guard.take() {
                    let _ = w.finalize();
                }
            }
            return Err(err);
        }
    }

    drop(stream);
    std::thread::sleep(std::time::Duration::from_millis(50));

    if let Ok(mut guard) = writer.lock() {
        if let Some(w) = guard.take() {
            w.finalize()
                .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
        }
    }

    Ok(())
}
