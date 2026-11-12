use rand::RngExt;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::f32::consts::PI;

pub const SPECTRUM_BARS_COUNT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizerStyle {
    SolidBars,      // 1. High-precision vertical gradient bars with peak hold caps
    ParticleSparks, // 2. Physics audio particle fountain & firework sparks
    BrailleWave,    // 3. Ultra-high resolution 2x4 sub-pixel Braille waveform
    AuroraPlasma,   // 4. Northern lights plasma field (inspired by ascii.rest)
    Donut3D,        // 5. Audio-reactive 3D spinning torus (inspired by ascii.rest)
    DoomFire,       // 6. Classic demoscene audio bonfire (inspired by ascii.rest)
    MatrixRain,     // 7. Cyber digital rain (inspired by ascii.rest)
    CosmicWarp,     // 8. 3D radial starburst hyperspace
}

impl VisualizerStyle {
    pub fn cycle(&self) -> Self {
        match self {
            VisualizerStyle::SolidBars => VisualizerStyle::ParticleSparks,
            VisualizerStyle::ParticleSparks => VisualizerStyle::BrailleWave,
            VisualizerStyle::BrailleWave => VisualizerStyle::AuroraPlasma,
            VisualizerStyle::AuroraPlasma => VisualizerStyle::Donut3D,
            VisualizerStyle::Donut3D => VisualizerStyle::DoomFire,
            VisualizerStyle::DoomFire => VisualizerStyle::MatrixRain,
            VisualizerStyle::MatrixRain => VisualizerStyle::CosmicWarp,
            VisualizerStyle::CosmicWarp => VisualizerStyle::SolidBars,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            VisualizerStyle::SolidBars => "Gradient Bars",
            VisualizerStyle::ParticleSparks => "Particle Sparks",
            VisualizerStyle::BrailleWave => "Braille Wave (High-Res)",
            VisualizerStyle::AuroraPlasma => "Aurora Plasma",
            VisualizerStyle::Donut3D => "3D Donut",
            VisualizerStyle::DoomFire => "Demoscene Fire",
            VisualizerStyle::MatrixRain => "Matrix Rain",
            VisualizerStyle::CosmicWarp => "Cosmic Warp",
        }
    }
}

// Particle for ParticleSparks
#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub glyph: char,
    pub color: Color,
}

// Star for CosmicWarp
#[derive(Debug, Clone)]
pub struct WarpStar {
    pub angle: f32,
    pub dist: f32,
    pub speed: f32,
}

// Column for MatrixRain
#[derive(Debug, Clone)]
pub struct MatrixColumn {
    pub head_y: f32,
    pub speed: f32,
    pub length: usize,
    pub chars: Vec<char>,
}

// High-resolution 2x4 Sub-pixel Braille Canvas
pub struct BrailleCanvas {
    pub width: usize,   // character columns
    pub height: usize,  // character rows
    pub pixel_w: usize, // width * 2
    pub pixel_h: usize, // height * 4
    dots: Vec<u8>,
    colors: Vec<Color>,
}

impl BrailleCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            pixel_w: width * 2,
            pixel_h: height * 4,
            dots: vec![0; size],
            colors: vec![Color::Reset; size],
        }
    }

    pub fn set_pixel(&mut self, px: usize, py: usize, color: Color) {
        if px >= self.pixel_w || py >= self.pixel_h {
            return;
        }

        let cx = px / 2;
        let cy = py / 4;
        let cell_idx = cy * self.width + cx;

        let sub_x = px % 2;
        let sub_y = py % 4;

        let bit = match (sub_x, sub_y) {
            (0, 0) => 0x01,
            (0, 1) => 0x02,
            (0, 2) => 0x04,
            (1, 0) => 0x08,
            (1, 1) => 0x10,
            (1, 2) => 0x20,
            (0, 3) => 0x40,
            (1, 3) => 0x80,
            _ => 0,
        };

        self.dots[cell_idx] |= bit;
        self.colors[cell_idx] = color;
    }

    pub fn draw_line(&mut self, mut x0: isize, mut y0: isize, x1: isize, y1: isize, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && (x0 as usize) < self.pixel_w && y0 >= 0 && (y0 as usize) < self.pixel_h {
                self.set_pixel(x0 as usize, y0 as usize, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::with_capacity(self.height);
        for cy in 0..self.height {
            let mut spans = Vec::new();
            let mut cur_str = String::new();
            let mut cur_style = Style::default();

            for cx in 0..self.width {
                let idx = cy * self.width + cx;
                let dot_mask = self.dots[idx];
                let ch = if dot_mask == 0 {
                    ' '
                } else {
                    char::from_u32(0x2800 + dot_mask as u32).unwrap_or(' ')
                };

                let color = self.colors[idx];
                let style = Style::default().fg(color);

                if style == cur_style {
                    cur_str.push(ch);
                } else {
                    if !cur_str.is_empty() {
                        spans.push(Span::styled(cur_str, cur_style));
                        cur_str = String::new();
                    }
                    cur_style = style;
                    cur_str.push(ch);
                }
            }
            if !cur_str.is_empty() {
                spans.push(Span::styled(cur_str, cur_style));
            }
            lines.push(Line::from(spans));
        }
        lines
    }
}

pub struct SpectrumVisualizer {
    pub bars: Vec<f32>,
    pub peaks: Vec<f32>,
    pub style: VisualizerStyle,
    pub particles: Vec<Particle>,
    pub warp_stars: Vec<WarpStar>,
    pub matrix_cols: Vec<MatrixColumn>,
    pub fire_buffer: Vec<Vec<u8>>,
    donut_a: f32,
    donut_b: f32,
    tick_count: u64,
}

impl SpectrumVisualizer {
    pub fn new() -> Self {
        let mut warp_stars = Vec::with_capacity(80);
        let mut rng = rand::rng();
        for _ in 0..80 {
            warp_stars.push(WarpStar {
                angle: rng.random_range(0.0..(2.0 * PI)),
                dist: rng.random_range(0.05..1.0),
                speed: rng.random_range(0.015..0.045),
            });
        }

        Self {
            bars: vec![0.0; SPECTRUM_BARS_COUNT],
            peaks: vec![0.0; SPECTRUM_BARS_COUNT],
            style: VisualizerStyle::SolidBars,
            particles: Vec::with_capacity(140),
            warp_stars,
            matrix_cols: Vec::new(),
            fire_buffer: Vec::new(),
            donut_a: 0.0,
            donut_b: 0.0,
            tick_count: 0,
        }
    }

    pub fn cycle_style(&mut self) -> VisualizerStyle {
        self.style = self.style.cycle();
        self.style
    }

    pub fn update(&mut self, is_playing: bool) {
        self.tick_count = self.tick_count.wrapping_add(1);
        let mut rng = rand::rng();

        // 1. Update frequency bars & peaks
        for i in 0..SPECTRUM_BARS_COUNT {
            if is_playing {
                let norm = i as f32 / SPECTRUM_BARS_COUNT as f32;
                let bass_bias = (1.0 - norm).powf(1.4);
                let mid_bump = (-((norm - 0.45) * 4.0).powi(2)).exp() * 0.7;
                let wave = ((self.tick_count as f32 * 0.25 + i as f32 * 0.6).sin() + 1.0) * 0.5;
                let wave2 = ((self.tick_count as f32 * 0.12 - i as f32 * 0.4).cos() + 1.0) * 0.5;

                let energy_impulse: f32 = rng.random_range(0.0..1.0);
                let target = (bass_bias * 0.55 + mid_bump * 0.35 + wave * 0.25 + wave2 * 0.2)
                    * (0.6 + energy_impulse * 0.5);

                let attack = 0.55;
                let decay = 0.25;
                if target > self.bars[i] {
                    self.bars[i] += (target - self.bars[i]) * attack;
                } else {
                    self.bars[i] -= (self.bars[i] - target) * decay;
                }
            } else {
                self.bars[i] = (self.bars[i] * 0.85 - 0.02).max(0.0);
            }

            self.bars[i] = self.bars[i].clamp(0.0, 1.0);

            if self.bars[i] >= self.peaks[i] {
                self.peaks[i] = self.bars[i];
            } else {
                self.peaks[i] = (self.peaks[i] - 0.025).max(0.0);
            }
        }

        // 2. Update style-specific physics
        self.update_particles(is_playing);
        self.update_warp_stars(is_playing);
        self.update_donut(is_playing);
    }

    fn update_donut(&mut self, is_playing: bool) {
        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;
        let speed = if is_playing { 0.06 + avg_energy * 0.12 } else { 0.02 };
        self.donut_a += speed;
        self.donut_b += speed * 0.6;
    }

    fn update_particles(&mut self, is_playing: bool) {
        let mut rng = rand::rng();

        // Spawn particles based on audio energy
        if is_playing {
            for (idx, &val) in self.bars.iter().enumerate() {
                if val > 0.30 && rng.random_range(0.0..1.0) < (val * 0.75) {
                    if self.particles.len() < 140 {
                        let norm = idx as f32 / SPECTRUM_BARS_COUNT as f32;
                        let color = if norm < 0.33 {
                            Color::Rgb(0, 240, 255) // Cyan
                        } else if norm < 0.66 {
                            Color::Rgb(255, 60, 220) // Magenta
                        } else {
                            Color::Rgb(255, 230, 40) // Yellow
                        };

                        let glyphs = if norm < 0.33 {
                            ['✦', '★', '✸', '✹']
                        } else if norm < 0.66 {
                            ['✧', '*', '✦', 'o']
                        } else {
                            ['·', '°', '.', '*']
                        };
                        let glyph = glyphs[rng.random_range(0..glyphs.len())];

                        let spread = rng.random_range(-0.4..0.4);
                        let up_speed = rng.random_range(0.4..1.1) * (0.6 + val * 0.6);

                        self.particles.push(Particle {
                            x: norm,
                            y: val,
                            vx: spread * 0.045,
                            vy: up_speed * 0.13,
                            life: 1.0,
                            max_life: rng.random_range(14.0..32.0),
                            glyph,
                            color,
                        });
                    }
                }
            }
        }

        let decay_step = if is_playing { 1.0 } else { 2.5 };
        for p in self.particles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            p.vy -= 0.007; // gravity
            p.vx *= 0.96;  // air drag
            p.life -= decay_step / p.max_life;

            if p.life < 0.35 {
                p.glyph = '·';
            } else if p.life < 0.65 {
                p.glyph = '•';
            }
        }

        self.particles.retain(|p| p.life > 0.0 && p.y >= 0.0 && p.x >= 0.0 && p.x <= 1.0);
    }

    fn update_warp_stars(&mut self, is_playing: bool) {
        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;
        let speed_mult = if is_playing { 0.8 + avg_energy * 2.4 } else { 0.2 };

        let mut rng = rand::rng();
        for star in self.warp_stars.iter_mut() {
            star.dist += star.speed * speed_mult;
            if star.dist > 1.05 {
                star.dist = rng.random_range(0.02..0.12);
                star.angle = rng.random_range(0.0..(2.0 * PI));
                star.speed = rng.random_range(0.015..0.045);
            }
        }
    }

    pub fn render(&mut self, width: usize, height: usize) -> Vec<Line<'static>> {
        if width == 0 || height == 0 {
            return vec![Line::raw(""); height];
        }

        match self.style {
            VisualizerStyle::SolidBars => self.render_solid_bars(width, height),
            VisualizerStyle::ParticleSparks => self.render_particle_sparks(width, height),
            VisualizerStyle::BrailleWave => self.render_braille_wave(width, height),
            VisualizerStyle::AuroraPlasma => self.render_aurora_plasma(width, height),
            VisualizerStyle::Donut3D => self.render_donut_3d(width, height),
            VisualizerStyle::DoomFire => self.render_doom_fire(width, height),
            VisualizerStyle::MatrixRain => self.render_matrix_rain(width, height),
            VisualizerStyle::CosmicWarp => self.render_cosmic_warp(width, height),
        }
    }

    // 1. SOLID GRADIENT BARS (Multi-height block resolution + peak caps)
    fn render_solid_bars(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        const BLOCKS: [char; 9] = [' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let bars_count = SPECTRUM_BARS_COUNT.min(width / 2);
        if bars_count == 0 {
            return vec![Line::raw(""); height];
        }

        let padding = (width.saturating_sub(bars_count * 2)) / 2;
        let mut grid = vec![vec![(' ', Style::default()); width]; height];

        for (bar_idx, &val) in self.bars.iter().take(bars_count).enumerate() {
            let x = padding + bar_idx * 2;
            let norm_bar = bar_idx as f32 / bars_count as f32;

            // Vibrant TrueColor gradient
            let color = if norm_bar < 0.35 {
                Color::Rgb(0, 220, 255)
            } else if norm_bar < 0.70 {
                Color::Rgb(240, 50, 220)
            } else {
                Color::Rgb(255, 230, 30)
            };
            let style = Style::default().fg(color);

            let total_substeps = height * 8;
            let filled_substeps = (val * total_substeps as f32).round() as usize;

            for row in 0..height {
                let row_inv = height - 1 - row;
                let row_start = row_inv * 8;
                let ch = if filled_substeps >= row_start + 8 {
                    '█'
                } else if filled_substeps > row_start {
                    let rem = filled_substeps - row_start;
                    BLOCKS[rem.min(8)]
                } else {
                    ' '
                };

                if ch != ' ' && x < width {
                    grid[row][x] = (ch, style);
                }
            }

            // Floating peak cap
            let peak_val = self.peaks[bar_idx];
            let peak_row_inv = (peak_val * (height as f32 - 0.5)).round() as usize;
            let peak_row = height.saturating_sub(1).saturating_sub(peak_row_inv);
            if peak_row < height && x < width && grid[peak_row][x].0 == ' ' {
                grid[peak_row][x] = ('▔', Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
            }
        }

        grid_to_lines(grid)
    }

    // 2. PARTICLE SPARKS (Physics particle fountains & fireworks)
    fn render_particle_sparks(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut grid = vec![vec![(' ', Style::default()); width]; height];

        // Bottom baseline audio wave
        for x in 0..width {
            let norm = x as f32 / width as f32;
            let bar_idx = ((norm * (SPECTRUM_BARS_COUNT - 1) as f32).round() as usize).min(SPECTRUM_BARS_COUNT - 1);
            let val = self.bars[bar_idx];
            let base_h = (val * (height as f32 * 0.40)).round() as usize;

            let color = if norm < 0.33 {
                Color::Rgb(0, 210, 240)
            } else if norm < 0.66 {
                Color::Rgb(240, 50, 210)
            } else {
                Color::Rgb(255, 220, 40)
            };

            for y in 0..base_h {
                let row = height.saturating_sub(1 + y);
                if row < height {
                    let ch = if y + 1 == base_h { '▄' } else { '█' };
                    grid[row][x] = (ch, Style::default().fg(color));
                }
            }
        }

        // Overlay flying particles
        for p in &self.particles {
            let x = (p.x * (width.saturating_sub(1)) as f32).round() as usize;
            let row_offset = (p.y * (height as f32 - 1.0)).round() as usize;
            let row = height.saturating_sub(1 + row_offset);

            if row < height && x < width {
                let style = if p.life > 0.6 {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(p.color)
                };
                grid[row][x] = (p.glyph, style);
            }
        }

        grid_to_lines(grid)
    }

    // 3. BRAILLE WAVEFORM (2x4 Sub-pixel continuous connected oscilloscope)
    fn render_braille_wave(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut canvas = BrailleCanvas::new(width, height);
        let pixel_w = canvas.pixel_w;
        let pixel_h = canvas.pixel_h;
        let mid_py = pixel_h as f32 * 0.5;

        let t = self.tick_count as f32 * 0.18;
        let mut prev_pt: Option<(isize, isize)> = None;

        for px in 0..pixel_w {
            let norm = px as f32 / pixel_w as f32;
            let bar_idx = ((norm * (SPECTRUM_BARS_COUNT - 1) as f32).round() as usize).min(SPECTRUM_BARS_COUNT - 1);
            let amp = self.bars[bar_idx];

            let sine1 = (norm * 16.0 + t).sin() * amp * (pixel_h as f32 * 0.38);
            let sine2 = (norm * 28.0 - t * 1.4).cos() * (amp * 0.7) * (pixel_h as f32 * 0.22);
            let py = (mid_py + sine1 + sine2).clamp(0.0, (pixel_h - 1) as f32) as isize;

            let color = if norm < 0.33 {
                Color::Rgb(0, 220, 255)
            } else if norm < 0.66 {
                Color::Rgb(240, 50, 220)
            } else {
                Color::Rgb(255, 230, 40)
            };

            let cur_pt = (px as isize, py);
            if let Some((x0, y0)) = prev_pt {
                canvas.draw_line(x0, y0, cur_pt.0, cur_pt.1, color);
            } else {
                canvas.set_pixel(px, py as usize, color);
            }
            prev_pt = Some(cur_pt);

            // Audio envelope mirror line if amp is strong
            if amp > 0.45 {
                let mirror_py = (mid_py - (sine1 + sine2)).clamp(0.0, (pixel_h - 1) as f32) as usize;
                canvas.set_pixel(px, mirror_py, Color::Rgb(120, 100, 200));
            }
        }

        canvas.to_lines()
    }

    // 4. AURORA PLASMA (Inspired by ascii.rest aurora)
    fn render_aurora_plasma(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut grid = vec![vec![(' ', Style::default()); width]; height];
        let t = self.tick_count as f32 * 0.12;

        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;
        let turbulence = 1.0 + avg_energy * 2.2;

        const AURORA_GLYPHS: [char; 8] = [' ', '·', '≈', '░', '▒', '▓', '█', '█'];

        for y in 0..height {
            let ny = y as f32 / height as f32;
            for x in 0..width {
                let nx = x as f32 / width as f32;

                let v1 = (nx * 5.0 * turbulence + t * 0.7).sin();
                let v2 = (ny * 6.0 - t * 0.5 + v1).sin();
                let v3 = ((nx + ny) * 4.0 + t * 0.4).cos();
                let val = ((v1 + v2 + v3 + 3.0) / 6.0).clamp(0.0, 1.0);

                // Blend emerald green -> cyan -> electric lavender
                let r = ((val * 180.0) as u8).saturating_add((avg_energy * 50.0) as u8);
                let g = (140.0 + (1.0 - val) * 115.0) as u8;
                let b = ((val * 240.0) as u8).max(80);

                let g_idx = (val * 7.0).round() as usize;
                let ch = AURORA_GLYPHS[g_idx.min(7)];

                grid[y][x] = (ch, Style::default().fg(Color::Rgb(r, g, b)));
            }
        }

        grid_to_lines(grid)
    }

    // 5. 3D ROTATING DONUT (Inspired by ascii.rest donut, audio-reactive)
    fn render_donut_3d(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut b = vec![' '; width * height];
        let mut z = vec![0.0f32; width * height];
        let mut colors = vec![Color::Reset; width * height];

        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;

        // Radius pulsates to music
        let r1 = 1.0 + avg_energy * 0.4;
        let r2 = 1.9 + avg_energy * 0.6;
        let k2 = 5.0;
        let k1 = (width as f32) * k2 * 3.0 / (8.0 * (r1 + r2));

        let cos_a = self.donut_a.cos();
        let sin_a = self.donut_a.sin();
        let cos_b = self.donut_b.cos();
        let sin_b = self.donut_b.sin();

        let mut theta = 0.0f32;
        while theta < 2.0 * PI {
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            let mut phi = 0.0f32;
            while phi < 2.0 * PI {
                let cos_phi = phi.cos();
                let sin_phi = phi.sin();

                let circ_x = r2 + r1 * cos_theta;
                let circ_y = r1 * sin_theta;

                let x = circ_x * (cos_b * cos_phi + sin_a * sin_b * sin_phi) - circ_y * cos_a * sin_b;
                let y = circ_x * (sin_b * cos_phi - sin_a * cos_b * sin_phi) + circ_y * cos_a * cos_b;
                let z_depth = k2 + cos_a * circ_x * sin_phi + circ_y * sin_a;
                let ooz = 1.0 / z_depth;

                let xp = ((width as f32 / 2.0) + k1 * ooz * x).round() as isize;
                let yp = ((height as f32 / 2.0) - (k1 * 0.5) * ooz * y).round() as isize;

                let l = cos_phi * cos_theta * sin_b
                    - cos_a * cos_theta * sin_phi
                    - sin_a * sin_theta
                    + cos_b * (cos_a * sin_theta - cos_theta * sin_a * sin_phi);

                if l > 0.0 && xp >= 0 && (xp as usize) < width && yp >= 0 && (yp as usize) < height {
                    let idx = yp as usize * width + xp as usize;
                    if ooz > z[idx] {
                        z[idx] = ooz;
                        const SHADES: &[u8] = b".,-~:;=!*#$@";
                        let lum_idx = ((l * 8.0).round() as usize).min(SHADES.len() - 1);
                        b[idx] = SHADES[lum_idx] as char;

                        // Neon synthwave depth lighting
                        let color_r = (180.0 * l + 40.0) as u8;
                        let color_g = (60.0 * l + 20.0) as u8;
                        let color_b = (240.0 * (1.0 - ooz * 2.0)).clamp(80.0, 255.0) as u8;
                        colors[idx] = Color::Rgb(color_r, color_g, color_b);
                    }
                }
                phi += 0.07;
            }
            theta += 0.05;
        }

        let mut lines = Vec::with_capacity(height);
        for row in 0..height {
            let mut spans = Vec::new();
            let mut cur_str = String::new();
            let mut cur_color = Color::Reset;

            for col in 0..width {
                let idx = row * width + col;
                let ch = b[idx];
                let color = colors[idx];

                if color == cur_color {
                    cur_str.push(ch);
                } else {
                    if !cur_str.is_empty() {
                        spans.push(Span::styled(cur_str, Style::default().fg(cur_color)));
                        cur_str = String::new();
                    }
                    cur_color = color;
                    cur_str.push(ch);
                }
            }
            if !cur_str.is_empty() {
                spans.push(Span::styled(cur_str, Style::default().fg(cur_color)));
            }
            lines.push(Line::from(spans));
        }

        lines
    }

    // 6. DEMOSCENE DOOM FIRE (Classic demoscene fire algorithm fed by audio bars)
    fn render_doom_fire(&mut self, width: usize, height: usize) -> Vec<Line<'static>> {
        if self.fire_buffer.len() != height || self.fire_buffer.first().map_or(0, |r| r.len()) != width {
            self.fire_buffer = vec![vec![0u8; width]; height];
        }

        let mut rng = rand::rng();

        // Feed bottom row with audio spectrum bars
        for x in 0..width {
            let norm = x as f32 / width as f32;
            let bar_idx = ((norm * (SPECTRUM_BARS_COUNT - 1) as f32).round() as usize).min(SPECTRUM_BARS_COUNT - 1);
            let val = self.bars[bar_idx];

            let flicker = rng.random_range(0.85..1.15);
            let heat = (val * 255.0 * flicker).clamp(0.0, 255.0) as u8;
            self.fire_buffer[height - 1][x] = heat;
        }

        // Fire propagation upward
        for y in 1..height {
            for x in 0..width {
                let src_y = y;
                let decay = rng.random_range(12..28);
                let spread: i32 = rng.random_range(-1..=1);
                let src_x = (x as isize + spread as isize).clamp(0, width as isize - 1) as usize;

                let below = self.fire_buffer[src_y][src_x];
                let new_heat = below.saturating_sub(decay);
                self.fire_buffer[y - 1][x] = new_heat;
            }
        }

        // Palette mapping: Black -> Deep Red -> Orange -> Gold -> White
        const FIRE_CHARS: [char; 8] = [' ', '.', ':', '+', '*', '░', '▓', '█'];
        let mut grid = vec![vec![(' ', Style::default()); width]; height];

        for y in 0..height {
            for x in 0..width {
                let heat = self.fire_buffer[y][x];
                if heat < 10 {
                    grid[y][x] = (' ', Style::default());
                    continue;
                }

                let ch_idx = (heat as usize * 7 / 255).min(7);
                let ch = FIRE_CHARS[ch_idx];

                let (r, g, b) = if heat < 80 {
                    (heat.saturating_mul(2), 0, 0)
                } else if heat < 160 {
                    (255, ((heat - 80) as f32 * 1.5) as u8, 0)
                } else if heat < 220 {
                    (255, 120 + ((heat - 160) as f32 * 1.8) as u8, 20)
                } else {
                    (255, 240, 180 + ((heat - 220) as f32 * 2.0) as u8)
                };

                grid[y][x] = (ch, Style::default().fg(Color::Rgb(r, g, b)));
            }
        }

        grid_to_lines(grid)
    }

    // 7. MATRIX DIGITAL RAIN (Inspired by ascii.rest matrix, audio-responsive)
    fn render_matrix_rain(&mut self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut rng = rand::rng();

        if self.matrix_cols.len() != width {
            self.matrix_cols.clear();
            const GLYPHS: &[char] = &['0', '1', '7', 'X', 'Z', '9', 'λ', 'Ψ', 'Ω', '§', '*', '#', '+', ':', '='];
            for _ in 0..width {
                let len = rng.random_range(4..14);
                let col_chars: Vec<char> = (0..len).map(|_| GLYPHS[rng.random_range(0..GLYPHS.len())]).collect();
                self.matrix_cols.push(MatrixColumn {
                    head_y: rng.random_range(-20.0..(height as f32)),
                    speed: rng.random_range(0.35..0.95),
                    length: len,
                    chars: col_chars,
                });
            }
        }

        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;
        let speed_boost = 1.0 + avg_energy * 2.0;

        let mut grid = vec![vec![(' ', Style::default()); width]; height];

        for (col_idx, col) in self.matrix_cols.iter_mut().enumerate() {
            col.head_y += col.speed * speed_boost;
            if col.head_y - (col.length as f32) > height as f32 {
                col.head_y = rng.random_range(-10.0..-1.0);
                col.speed = rng.random_range(0.35..0.95);
            }

            let hy = col.head_y.round() as isize;
            for seg in 0..col.length {
                let cy = hy - seg as isize;
                if cy >= 0 && (cy as usize) < height && col_idx < width {
                    let ch = col.chars[seg % col.chars.len()];
                    let style = if seg == 0 {
                        // Glowing leading head
                        Style::default()
                            .fg(Color::Rgb(220, 255, 240))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        // Fading trail
                        let fade = 1.0 - (seg as f32 / col.length as f32);
                        let g = (fade * 220.0 + 35.0) as u8;
                        Style::default().fg(Color::Rgb(0, g, (fade * 50.0) as u8))
                    };
                    grid[cy as usize][col_idx] = (ch, style);
                }
            }
        }

        grid_to_lines(grid)
    }

    // 8. COSMIC WARP (3D Radial Starburst Hyperspace)
    fn render_cosmic_warp(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut grid = vec![vec![(' ', Style::default()); width]; height];
        let cx = width as f32 * 0.5;
        let cy = height as f32 * 0.5;

        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;

        for star in &self.warp_stars {
            let rx = star.dist * (width as f32 * 0.48);
            let ry = star.dist * (height as f32 * 0.48);

            let x = (cx + star.angle.cos() * rx).round() as isize;
            let y = (cy + star.angle.sin() * ry).round() as isize;

            if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
                let ux = x as usize;
                let uy = y as usize;

                let norm_angle = (star.angle + PI) / (2.0 * PI);
                let color = if norm_angle < 0.33 {
                    Color::Rgb(0, 240, 255)
                } else if norm_angle < 0.66 {
                    Color::Rgb(255, 60, 220)
                } else {
                    Color::Rgb(255, 230, 40)
                };

                let (ch, style) = if star.dist > 0.85 {
                    ('━', Style::default().fg(color).add_modifier(Modifier::BOLD))
                } else if star.dist > 0.60 {
                    ('✦', Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                } else if star.dist > 0.35 {
                    ('★', Style::default().fg(color))
                } else if star.dist > 0.15 {
                    ('•', Style::default().fg(color))
                } else {
                    ('.', Style::default().fg(Color::DarkGray))
                };

                grid[uy][ux] = (ch, style);
            }
        }

        let core_x = cx.round() as usize;
        let core_y = cy.round() as usize;
        if core_y < height && core_x < width {
            let core_glyph = if avg_energy > 0.55 {
                '✹'
            } else if avg_energy > 0.30 {
                '●'
            } else {
                '◉'
            };
            grid[core_y][core_x] = (
                core_glyph,
                Style::default().fg(Color::Rgb(255, 230, 0)).add_modifier(Modifier::BOLD),
            );
        }

        grid_to_lines(grid)
    }
}

fn grid_to_lines(grid: Vec<Vec<(char, Style)>>) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(grid.len());
    for row in grid {
        let mut spans = Vec::new();
        let mut cur_str = String::new();
        let mut cur_style = Style::default();

        for (ch, style) in row {
            if style == cur_style {
                cur_str.push(ch);
            } else {
                if !cur_str.is_empty() {
                    spans.push(Span::styled(cur_str, cur_style));
                    cur_str = String::new();
                }
                cur_style = style;
                cur_str.push(ch);
            }
        }
        if !cur_str.is_empty() {
            spans.push(Span::styled(cur_str, cur_style));
        }
        lines.push(Line::from(spans));
    }
    lines
}
