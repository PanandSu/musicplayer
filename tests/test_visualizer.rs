use musicplayer::visualizer::{SpectrumVisualizer, VisualizerStyle};

#[test]
fn test_all_visualizer_styles_render() {
    let mut vis = SpectrumVisualizer::new();

    let styles = [
        VisualizerStyle::SolidBars,
        VisualizerStyle::ParticleSparks,
        VisualizerStyle::BrailleWave,
        VisualizerStyle::AuroraPlasma,
        VisualizerStyle::Donut3D,
        VisualizerStyle::DoomFire,
        VisualizerStyle::MatrixRain,
        VisualizerStyle::CosmicWarp,
    ];

    // Simulate audio playback to charge bars, particles, and stars
    for _ in 0..20 {
        vis.update(true);
    }

    for style in styles {
        vis.style = style;
        assert_eq!(vis.style, style);
        assert!(!vis.style.display_name().is_empty());

        // Test multiple terminal aspect ratios
        for (w, h) in [(40, 10), (80, 15), (120, 25), (20, 4)] {
            let lines = vis.render(w, h);
            assert_eq!(lines.len(), h, "Rendered lines must match height {}", h);
        }
    }
}

#[test]
fn test_visualizer_style_cycle() {
    let mut vis = SpectrumVisualizer::new();
    assert_eq!(vis.style, VisualizerStyle::SolidBars);

    assert_eq!(vis.cycle_style(), VisualizerStyle::ParticleSparks);
    assert_eq!(vis.cycle_style(), VisualizerStyle::BrailleWave);
    assert_eq!(vis.cycle_style(), VisualizerStyle::AuroraPlasma);
    assert_eq!(vis.cycle_style(), VisualizerStyle::Donut3D);
    assert_eq!(vis.cycle_style(), VisualizerStyle::DoomFire);
    assert_eq!(vis.cycle_style(), VisualizerStyle::MatrixRain);
    assert_eq!(vis.cycle_style(), VisualizerStyle::CosmicWarp);
    assert_eq!(vis.cycle_style(), VisualizerStyle::SolidBars);
}

#[test]
fn test_particle_physics() {
    let mut vis = SpectrumVisualizer::new();
    vis.style = VisualizerStyle::ParticleSparks;

    for _ in 0..30 {
        vis.update(true);
    }
    assert!(!vis.particles.is_empty(), "Particles should spawn when playing");

    for _ in 0..60 {
        vis.update(false);
    }
    assert!(
        vis.particles.len() < 5,
        "Particles should decay when paused/stopped"
    );
}
