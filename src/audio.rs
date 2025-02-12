use std::sync::{Arc, Mutex};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{Sample, FromSample};

use crate::io::sound::AudioPlayer;


pub struct CpalPlayer {
    buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    sample_rate: u32,
}

impl CpalPlayer {
    pub fn get() -> Option<(CpalPlayer, cpal::Stream)> {
        let device = match cpal::default_host().default_output_device() {
            Some(e) => e,
            None => return None,
        };

        // We want a config with:
        // chanels = 2
        // SampleFormat F32
        // Rate at around 44100

        let wanted_samplerate = cpal::SampleRate(44100);
        let supported_configs = match device.supported_output_configs() {
            Ok(e) => e,
            Err(_) => return None,
        };
        let mut supported_config = None;
        for f in supported_configs {
            if f.channels() == 2 && f.sample_format() == cpal::SampleFormat::F32 {
                if f.min_sample_rate() <= wanted_samplerate && wanted_samplerate <= f.max_sample_rate() {
                    supported_config = Some(f.with_sample_rate(wanted_samplerate));
                }
                else {
                    supported_config = Some(f.with_max_sample_rate());
                }
                break;
            }
        }
        if supported_config.is_none() {
            return None;
        }

        let selected_config = supported_config.unwrap();

        let sample_format = selected_config.sample_format();
        let config : cpal::StreamConfig = selected_config.into();

        let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);

        let shared_buffer = Arc::new(Mutex::new(Vec::new()));
        let stream_buffer = shared_buffer.clone();

        let player = CpalPlayer {
            buffer: shared_buffer,
            sample_rate: config.sample_rate.0,
        };

        let stream = match sample_format {
            cpal::SampleFormat::I8 => device.build_output_stream(&config, move|data: &mut [i8], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::I16 => device.build_output_stream(&config, move|data: &mut [i16], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::I32 => device.build_output_stream(&config, move|data: &mut [i32], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::I64 => device.build_output_stream(&config, move|data: &mut [i64], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::U8 => device.build_output_stream(&config, move|data: &mut [u8], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::U16 => device.build_output_stream(&config, move|data: &mut [u16], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::U32 => device.build_output_stream(&config, move|data: &mut [u32], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::U64 => device.build_output_stream(&config, move|data: &mut [u64], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::F32 => device.build_output_stream(&config, move|data: &mut [f32], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            cpal::SampleFormat::F64 => device.build_output_stream(&config, move|data: &mut [f64], _callback_info: &cpal::OutputCallbackInfo| cpal_thread(data, &stream_buffer), err_fn, None),
            sf => panic!("Unsupported sample format {}", sf),
        }.unwrap();

        stream.play().unwrap();

        Some((player, stream))
    }
}

fn cpal_thread<T: Sample + FromSample<f32>>(outbuffer: &mut[T], audio_buffer: &Arc<Mutex<Vec<(f32, f32)>>>) {
    let mut inbuffer = audio_buffer.lock().unwrap();
    let outlen =  ::std::cmp::min(outbuffer.len() / 2, inbuffer.len());
    for (i, (in_l, in_r)) in inbuffer.drain(..outlen).enumerate() {
        outbuffer[i*2] = T::from_sample(in_l);
        outbuffer[i*2+1] = T::from_sample(in_r);
    }
}

impl AudioPlayer for CpalPlayer {
    fn play(&mut self, buf_left: &[f32], buf_right: &[f32]) {
        debug_assert!(buf_left.len() == buf_right.len());

        let mut buffer = self.buffer.lock().unwrap();

        for (l, r) in buf_left.iter().zip(buf_right) {
            if buffer.len() > self.sample_rate as usize {
                // Do not fill the buffer with more than 1 second of data
                // This speeds up the resync after the turning on and off the speed limiter
                return
            }
            buffer.push((*l, *r));
        }
    }

    fn samples_rate(&self) -> u32 {
        self.sample_rate
    }

    fn underflowed(&self) -> bool {
        (*self.buffer.lock().unwrap()).len() == 0
    }
}

