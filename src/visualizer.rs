use rand::RngExt;

pub const SPECTRUM_BARS_COUNT: usize = 28;

pub struct SpectrumVisualizer {
    pub bars: Vec<f32>,
    pub peaks: Vec<f32>,
    #[allow(dead_code)]
    pub velocities: Vec<f32>,
    tick_count: u64,
}

impl SpectrumVisualizer {
    pub fn new() -> Self {
        Self {
            bars: vec![0.0; SPECTRUM_BARS_COUNT],
            peaks: vec![0.0; SPECTRUM_BARS_COUNT],
            velocities: vec![0.0; SPECTRUM_BARS_COUNT],
            tick_count: 0,
        }
    }

    pub fn update(&mut self, is_playing: bool) {
        self.tick_count = self.tick_count.wrapping_add(1);
        let mut rng = rand::rng();

        for i in 0..SPECTRUM_BARS_COUNT {
            if is_playing {
                // Harmonic weighting: bass bars have higher amplitude and punch,
                // mid bars have rhythmic variation, highs shimmer with fast jitter.
                let norm = i as f32 / SPECTRUM_BARS_COUNT as f32;
                let bass_bias = (1.0 - norm).powf(1.4);
                let mid_bump = (-((norm - 0.45) * 4.0).powi(2)).exp() * 0.7;
                let wave = ((self.tick_count as f32 * 0.25 + i as f32 * 0.6).sin() + 1.0) * 0.5;
                let wave2 = ((self.tick_count as f32 * 0.12 - i as f32 * 0.4).cos() + 1.0) * 0.5;

                let energy_impulse: f32 = rng.random_range(0.0..1.0);
                let target = (bass_bias * 0.55 + mid_bump * 0.35 + wave * 0.25 + wave2 * 0.2)
                    * (0.6 + energy_impulse * 0.5);

                // Smooth interpolation towards target
                let attack = 0.55;
                let decay = 0.25;
                if target > self.bars[i] {
                    self.bars[i] += (target - self.bars[i]) * attack;
                } else {
                    self.bars[i] -= (self.bars[i] - target) * decay;
                }
            } else {
                // Smooth decay to zero when paused or stopped
                self.bars[i] = (self.bars[i] * 0.85 - 0.02).max(0.0);
            }

            self.bars[i] = self.bars[i].clamp(0.0, 1.0);

            // Update peak hold
            if self.bars[i] >= self.peaks[i] {
                self.peaks[i] = self.bars[i];
            } else {
                self.peaks[i] = (self.peaks[i] - 0.03).max(0.0);
            }
        }
    }

    pub fn get_block_character(val: f32) -> char {
        const BLOCKS: [char; 9] = [' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let idx = (val.clamp(0.0, 1.0) * 8.0).round() as usize;
        BLOCKS[idx.min(8)]
    }

    pub fn render_bar_column(&self, index: usize, max_height: usize) -> Vec<char> {
        if index >= self.bars.len() || max_height == 0 {
            return vec![' '; max_height];
        }

        let val = self.bars[index];
        let total_substeps = max_height * 8;
        let filled_substeps = (val * total_substeps as f32).round() as usize;

        let mut col = Vec::with_capacity(max_height);
        for row in (0..max_height).rev() {
            let row_start = row * 8;
            if filled_substeps >= row_start + 8 {
                col.push('█');
            } else if filled_substeps > row_start {
                let rem = filled_substeps - row_start;
                col.push(Self::get_block_character(rem as f32 / 8.0));
            } else {
                col.push(' ');
            }
        }
        col
    }
}
