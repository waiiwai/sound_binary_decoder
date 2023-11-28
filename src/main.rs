extern crate hound;

use num::Complex;
use hound::WavReader;
use std::f32::consts::PI;
use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reader = WavReader::open(&args[1]).expect("Failed to open WAV file");
    let mut out_file = File::create(&args[2]).unwrap();

    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;
    let _num_samples = reader.len() as usize;
    let num_channels = spec.channels as usize;

    let signal_split = 4;
    let window_size = (sample_rate as usize).min((sample_rate/(signal_split as f32)) as usize);
    let mut step_size = (sample_rate as usize).min(window_size);

    let data_f = &vec![128, 256, 2048];


    for _channel in 0..num_channels {
        let mut signal = vec![0; window_size];
        let mut counter = 0;
        let mut ix = 0;
        let mut header_count = 0;
        let mut data = false;
        let mut pos = 0;
        let mut bits = 0;
        let mut buf: u8 = 0;
        for sample in reader.samples::<i32>().filter_map(Result::ok) {
            signal.push(sample);
            counter += 1;
            if counter % window_size == 0 {
                ix += 1;
                let rslt = dtft(&signal, sample_rate as i32, data_f);
                let (max_index, _max) =
                    rslt.iter()
                        .enumerate()
                        .fold((usize::MIN, f32::MIN), |(i_a, a), (i_b, &b)| {
                            if b.norm() > a {
                                (i_b, b.norm())
                            } else {
                                (i_a, a)
                            }
                        });
                if max_index == data_f.len() - 1 {
                    header_count += 1;
                    if data {
                        break;
                    }
                }

                if header_count > 1 && ix % signal_split == pos {
                    if max_index == 0 || max_index == 1  {
                        data = true;
                        buf += 2_u8.pow(bits%8) * (max_index as u8);
                        bits += 1;
                    }
                    if data && bits%8 == 0 {
                        write!(out_file, "{}", buf as char);
                        buf = 0;
                    }
                }

                if header_count == 2 {
                    pos = ix % signal_split;
                }
                signal.drain(0..step_size);
                counter -= step_size;
            }
        }
    }
}

pub fn dtft(frames: &Vec<i32>, fs: i32, targets: &Vec<u32>) -> Vec<Complex<f32>> {
    let mut rslt: Vec<Complex<f32>> = Vec::new();
    targets.iter().for_each(|f| {
        let mut sigma: Complex<f32> = Complex::new(0.0, 0.0);
        for (k, xk) in frames.iter().enumerate() {
            let t: f32 = (k as f32) / (fs as f32);
            let w = 2.0 * PI * (*f as f32);
            let exp = Complex::new(0.0, -w * t).exp();
            sigma += (*xk as f32)* exp;
        }
        rslt.push(sigma);
    });
    return rslt;
}
