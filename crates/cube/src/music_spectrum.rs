#![doc = include_str!("../../../rfcs/009_music_spectrum.md")]

use crate::App;
use alloc::vec::Vec;
use embassy_time::Timer;
use embedded_graphics_core::{Pixel, pixelcolor::Rgb888};
use microfft::real::rfft_64;
use micromath::F32Ext;

/// 音乐频谱
pub struct MusicSpectrum;

impl MusicSpectrum {
    pub async fn run(app: &mut App<'_>) {
        app.ledc.clear();

        loop {
            // 1. 采样音频（ADC 暂用随机数据 stub）
            let mut samples = [0.0f32; 64];
            // TODO: 替换为 ADC 采样
            // 当前使用模拟数据：基于时间的正弦波组合
            for i in 0..64 {
                let t = i as f32 / 64.0;
                samples[i] = (t * 6.2832 * 3.0).sin() * 0.5
                    + (t * 6.2832 * 7.0).sin() * 0.3
                    + (t * 6.2832 * 12.0).sin() * 0.2;
            }

            // 2. FFT 计算
            let spectrum = Self::compute_fft(&mut samples);

            // 3. 映射到 LED 显示
            let pixels = Self::map_to_display(&spectrum);

            // 4. 显示
            app.ledc.clear();
            app.ledc.write_pixels(pixels);

            // 5. 下甩暂停
            app.check_pause().await;

            Timer::after_millis(50).await; // ~20 FPS
        }
    }

    /// 使用 microfft 计算 64 点实数 FFT
    fn compute_fft(samples: &mut [f32; 64]) -> [f32; 32] {
        let result = rfft_64(samples);

        // 计算幅度谱（取前 32 个频率分量）
        let mut magnitudes = [0.0f32; 32];
        for (i, c) in result.iter().enumerate() {
            magnitudes[i] = (c.re * c.re + c.im * c.im).sqrt();
        }
        magnitudes
    }

    /// 将频谱映射到 8x8 LED 显示
    fn map_to_display(spectrum: &[f32; 32]) -> Vec<Pixel<Rgb888>> {
        let num_bands = 8usize;
        let band_size = 32 / num_bands; // 每个频段 4 个频率分量
        let mut pixels = Vec::new();

        for band in 0..num_bands {
            // 计算每个频段的平均幅度
            let start = band * band_size;
            let end = start + band_size;
            let avg: f32 = spectrum[start..end].iter().sum::<f32>() / band_size as f32;

            // 映射到 0-8 高度（增益系数需实际调参）
            let height = ((avg * 8.0).min(8.0)).max(0.0) as i32;

            // 颜色渐变：低频绿色 -> 高频红色
            let r = ((band * 255) / num_bands) as u8;
            let g = (255 - (band * 255) / num_bands) as u8;

            for y in 0..height {
                if y < 8 {
                    pixels.push(Pixel(
                        (band as i32, 7 - y).into(), // 从底部向上画
                        Rgb888::new(r, g, 0),
                    ));
                }
            }
        }
        pixels
    }
}
