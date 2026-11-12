use rand::RngExt;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::f32::consts::PI;

pub const SPECTRUM_BARS_COUNT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizerStyle {
    SolidBars,      // Modern solid vertical gradient bars with peak caps
    ParticleSparks, // Floating audio sparks & particle fountain
    BrailleWave,    // Cyberpunk high-resolution Braille waveform matrix
    RetroEqualizer, // Vintage Hi-Fi segmented LED rack equalizer
    CosmicWarp,     // 3D radial starburst hyperspace
}

impl VisualizerStyle {
    pub fn cycle(&self) -> Self {
        match self {
            VisualizerStyle::SolidBars => VisualizerStyle::ParticleSparks,
            VisualizerStyle::ParticleSparks => VisualizerStyle::BrailleWave,
            VisualizerStyle::BrailleWave => VisualizerStyle::RetroEqualizer,
            VisualizerStyle::RetroEqualizer => VisualizerStyle::CosmicWarp,
            VisualizerStyle::CosmicWarp => VisualizerStyle::SolidBars,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            VisualizerStyle::SolidBars => "Gradient Bars",
            VisualizerStyle::ParticleSparks => "Particle Sparks",
            VisualizerStyle::BrailleWave => "Braille Wave",
            VisualizerStyle::RetroEqualizer => "Retro Hi-Fi EQ",
            VisualizerStyle::CosmicWarp => "Cosmic Warp",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub glyph: char,
    pub color_idx: usize,
}

#[derive(Debug, Clone)]
pub struct WarpStar {
    pub angle: f32,
    pub dist: f32,
    pub speed: f32,
    pub z: f32,
}

pub struct SpectrumVisualizer {
    pub bars: Vec<f32>,
    pub peaks: Vec<f32>,
    pub style: VisualizerStyle,
    pub particles: Vec<Particle>,
    pub warp_stars: Vec<WarpStar>,
    tick_count: u64,
}

impl SpectrumVisualizer {
    pub fn new() -> Self {
        let mut warp_stars = Vec::with_capacity(70);
        let mut rng = rand::rng();
        for _ in 0..70 {
            warp_stars.push(WarpStar {
                angle: rng.random_range(0.0..(2.0 * PI)),
                dist: rng.random_range(0.05..1.0),
                speed: rng.random_range(0.015..0.045),
                z: rng.random_range(0.2..1.0),
            });
        }

        Self {
            bars: vec![0.0; SPECTRUM_BARS_COUNT],
            peaks: vec![0.0; SPECTRUM_BARS_COUNT],
            style: VisualizerStyle::SolidBars,
            particles: Vec::with_capacity(120),
            warp_stars,
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

        // 2. Update particle physics for ParticleSparks
        self.update_particles(is_playing);

        // 3. Update warp stars for CosmicWarp
        self.update_warp_stars(is_playing);
    }

    fn update_particles(&mut self, is_playing: bool) {
        let mut rng = rand::rng();

        // Spawn new particles from audio bars when playing
        if is_playing {
            for (idx, &val) in self.bars.iter().enumerate() {
                // Higher values have higher probability of spawning
                if val > 0.35 && rng.random_range(0.0..1.0) < (val * 0.65) {
                    if self.particles.len() < 120 {
                        let norm = idx as f32 / SPECTRUM_BARS_COUNT as f32;
                        let color_idx = if norm < 0.33 {
                            0 // Cyan (bass)
                        } else if norm < 0.66 {
                            1 // Magenta (mids)
                        } else {
                            2 // Yellow (treble)
                        };

                        let glyphs = if norm < 0.33 {
                            ['✦', '★', '✸', '✹']
                        } else if norm < 0.66 {
                            ['✧', '*', '✦', 'o']
                        } else {
                            ['·', '°', '.', '*']
                        };
                        let glyph = glyphs[rng.random_range(0..glyphs.len())];

                        let spread = rng.random_range(-0.35..0.35);
                        let up_speed = rng.random_range(0.35..0.95) * (0.6 + val * 0.6);

                        self.particles.push(Particle {
                            x: norm,
                            y: val, // spawn at peak of bar
                            vx: spread * 0.04,
                            vy: up_speed * 0.12,
                            life: 1.0,
                            max_life: rng.random_range(12.0..28.0),
                            glyph,
                            color_idx,
                        });
                    }
                }
            }
        }

        // Update physics and lifespan
        let decay_step = if is_playing { 1.0 } else { 2.5 };
        for p in self.particles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            p.vy -= 0.006; // gravity pulls downward relative to bar height
            p.vx *= 0.96; // air drag
            p.life -= decay_step / p.max_life;

            // Change glyph as particle ages
            if p.life < 0.35 {
                p.glyph = '·';
            } else if p.life < 0.65 {
                p.glyph = '•';
            }
        }

        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0 && p.y >= 0.0 && p.x >= 0.0 && p.x <= 1.0);
    }

    fn update_warp_stars(&mut self, is_playing: bool) {
        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;
        let speed_mult = if is_playing {
            0.8 + avg_energy * 2.4
        } else {
            0.2
        };

        let mut rng = rand::rng();
        for star in self.warp_stars.iter_mut() {
            star.dist += star.speed * speed_mult;
            if star.dist > 1.05 {
                // Reset star back to center
                star.dist = rng.random_range(0.02..0.12);
                star.angle = rng.random_range(0.0..(2.0 * PI));
                star.speed = rng.random_range(0.015..0.045);
            }
        }
    }

    pub fn render(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        match self.style {
            VisualizerStyle::SolidBars => self.render_solid_bars(width, height),
            VisualizerStyle::ParticleSparks => self.render_particle_sparks(width, height),
            VisualizerStyle::BrailleWave => self.render_braille_wave(width, height),
            VisualizerStyle::RetroEqualizer => self.render_retro_equalizer(width, height),
            VisualizerStyle::CosmicWarp => self.render_cosmic_warp(width, height),
        }
    }

    // 1. SOLID BARS STYLE (with peak hold caps)
    fn render_solid_bars(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        const BLOCKS: [char; 9] = [' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let bars_count = SPECTRUM_BARS_COUNT.min(width / 2);
        if bars_count == 0 || height == 0 {
            return vec![Line::raw(""); height];
        }

        let padding = (width.saturating_sub(bars_count * 2)) / 2;
        let mut grid = vec![vec![(' ', Style::default()); width]; height];

        for (bar_idx, &val) in self.bars.iter().take(bars_count).enumerate() {
            let x = padding + bar_idx * 2;
            let norm_bar = bar_idx as f32 / bars_count as f32;
            let color = if norm_bar < 0.35 {
                Color::Cyan
            } else if norm_bar < 0.70 {
                Color::Magenta
            } else {
                Color::Yellow
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

            // Draw floating peak cap
            let peak_val = self.peaks[bar_idx];
            let peak_row_inv = (peak_val * (height as f32 - 0.5)).round() as usize;
            let peak_row = height.saturating_sub(1).saturating_sub(peak_row_inv);
            if peak_row < height && x < width && grid[peak_row][x].0 == ' ' {
                grid[peak_row][x] = ('▔', Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
            }
        }

        grid_to_lines(grid)
    }

    // 2. PARTICLE SPARKS STYLE (ASCII sparks & fountain floating upward)
    fn render_particle_sparks(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        if width == 0 || height == 0 {
            return vec![Line::raw(""); height];
        }

        let mut grid = vec![vec![(' ', Style::default()); width]; height];

        // Bottom baseline audio wave
        for x in 0..width {
            let norm = x as f32 / width as f32;
            let bar_idx = ((norm * (SPECTRUM_BARS_COUNT - 1) as f32).round() as usize).min(SPECTRUM_BARS_COUNT - 1);
            let val = self.bars[bar_idx];
            let base_h = (val * (height as f32 * 0.45)).round() as usize;

            let color = if norm < 0.33 {
                Color::Cyan
            } else if norm < 0.66 {
                Color::Magenta
            } else {
                Color::Yellow
            };

            for y in 0..base_h {
                let row = height.saturating_sub(1 + y);
                if row < height {
                    let ch = if y + 1 == base_h { '▄' } else { '█' };
                    grid[row][x] = (ch, Style::default().fg(color));
                }
            }
        }

        // Overlay floating particles
        for p in &self.particles {
            let x = (p.x * (width.saturating_sub(1)) as f32).round() as usize;
            // Map p.y (0.0 to 1.0) into row height
            let row_offset = (p.y * (height as f32 - 1.0)).round() as usize;
            let row = height.saturating_sub(1 + row_offset);

            if row < height && x < width {
                let color = match p.color_idx {
                    0 => Color::Cyan,
                    1 => Color::Magenta,
                    _ => Color::Yellow,
                };
                let style = if p.life > 0.6 {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color)
                };
                grid[row][x] = (p.glyph, style);
            }
        }

        grid_to_lines(grid)
    }

    // 3. BRAILLE WAVEFORM STYLE (Cyberpunk high-density Braille dots)
    fn render_braille_wave(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        if width == 0 || height == 0 {
            return vec![Line::raw(""); height];
        }

        let mut grid = vec![vec![(' ', Style::default()); width]; height];
        let mid_y = height as f32 * 0.5;

        // Render dual-wave audio ripples using Braille dots
        for x in 0..width {
            let norm = x as f32 / width as f32;
            let bar_idx = ((norm * (SPECTRUM_BARS_COUNT - 1) as f32).round() as usize).min(SPECTRUM_BARS_COUNT - 1);
            let amp = self.bars[bar_idx];

            let t = self.tick_count as f32 * 0.2;
            let sine1 = (norm * 14.0 + t).sin() * amp * (height as f32 * 0.4);
            let sine2 = (norm * 22.0 - t * 1.5).cos() * (amp * 0.7) * (height as f32 * 0.3);

            let y_pos = (mid_y + sine1 + sine2).clamp(0.0, (height - 1) as f32);
            let row = y_pos.round() as usize;

            let color = if norm < 0.33 {
                Color::Cyan
            } else if norm < 0.66 {
                Color::Magenta
            } else {
                Color::Yellow
            };

            const BRAILLE_GLYPHS: [char; 8] = ['⠂', '⠒', '⠶', '⠛', '⠟', '⠿', '⣾', '⣿'];
            let g_idx = (amp * 7.0).round() as usize;
            let ch = BRAILLE_GLYPHS[g_idx.min(7)];

            if row < height {
                grid[row][x] = (ch, Style::default().fg(color).add_modifier(Modifier::BOLD));
                // Add vertical glow trail if amp is high
                if amp > 0.4 {
                    if row + 1 < height && grid[row + 1][x].0 == ' ' {
                        grid[row + 1][x] = ('⡀', Style::default().fg(color));
                    }
                    if row > 0 && grid[row - 1][x].0 == ' ' {
                        grid[row - 1][x] = ('⠉', Style::default().fg(color));
                    }
                }
            }
        }

        grid_to_lines(grid)
    }

    // 4. RETRO EQUALIZER STYLE (Vintage Hi-Fi segmented LED rack)
    fn render_retro_equalizer(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let bars_count = SPECTRUM_BARS_COUNT.min(width / 3);
        if bars_count == 0 || height == 0 {
            return vec![Line::raw(""); height];
        }

        let padding = (width.saturating_sub(bars_count * 3)) / 2;
        let mut grid = vec![vec![(' ', Style::default()); width]; height];

        for (idx, &val) in self.bars.iter().take(bars_count).enumerate() {
            let x = padding + idx * 3;
            let total_segments = height;
            let active_segments = (val * total_segments as f32).round() as usize;

            for seg in 0..total_segments {
                let row = height.saturating_sub(1 + seg);
                let seg_ratio = seg as f32 / total_segments as f32;

                // Vintage LED colors: Green (normal) -> Yellow (warning) -> Red (peak clip)
                let color = if seg_ratio < 0.55 {
                    Color::Green
                } else if seg_ratio < 0.80 {
                    Color::Yellow
                } else {
                    Color::Red
                };

                let is_lit = seg < active_segments;
                let ch = if is_lit { '▓' } else { '░' };
                let style = if is_lit {
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(40, 45, 50))
                };

                if row < height && x + 1 < width {
                    grid[row][x] = (ch, style);
                    grid[row][x + 1] = (ch, style);
                }
            }

            // Peak LED cap
            let peak_seg = (self.peaks[idx] * (total_segments as f32 - 0.5)).round() as usize;
            let peak_row = height.saturating_sub(1 + peak_seg);
            if peak_row < height && x + 1 < width {
                let peak_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
                grid[peak_row][x] = ('━', peak_style);
                grid[peak_row][x + 1] = ('━', peak_style);
            }
        }

        grid_to_lines(grid)
    }

    // 5. COSMIC WARP STYLE (3D Radial Starburst Hyperspace)
    fn render_cosmic_warp(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        if width == 0 || height == 0 {
            return vec![Line::raw(""); height];
        }

        let mut grid = vec![vec![(' ', Style::default()); width]; height];
        let cx = width as f32 * 0.5;
        let cy = height as f32 * 0.5;

        let avg_energy = self.bars.iter().copied().sum::<f32>() / SPECTRUM_BARS_COUNT as f32;

        // Render radial warp stars
        for star in &self.warp_stars {
            // Adjust aspect ratio (terminal chars are roughly twice as tall as wide)
            let rx = star.dist * (width as f32 * 0.48);
            let ry = star.dist * (height as f32 * 0.48);

            let x = (cx + star.angle.cos() * rx).round() as isize;
            let y = (cy + star.angle.sin() * ry).round() as isize;

            if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
                let ux = x as usize;
                let uy = y as usize;

                let norm_angle = (star.angle + PI) / (2.0 * PI);
                let color = if norm_angle < 0.33 {
                    Color::Cyan
                } else if norm_angle < 0.66 {
                    Color::Magenta
                } else {
                    Color::Yellow
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

        // Center pulsating audio core
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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
