use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;

#[test]
fn test_rodio_playback() {
    let builder = rodio::stream::DeviceSinkBuilder::from_default_device().unwrap();
    let mut device_sink = builder
        .with_error_callback(|_err| {})
        .open_sink_or_fallback()
        .unwrap();
    device_sink.log_on_drop(false);
    if let Ok(file) = File::open("As the World Falls Down - Aaron Richards.flac") {
        let player = rodio::stream::play(device_sink.mixer(), BufReader::new(file)).unwrap();
        player.pause();
        assert!(player.is_paused());
        player.play();
        assert!(!player.is_paused());
        let _ = player.try_seek(Duration::from_secs(2));
        player.set_volume(0.8);
        assert!((player.volume() - 0.8).abs() < 0.01);
    }
}

#[test]
fn test_lofty_metadata() {
    if let Ok(probe) = Probe::open("As the World Falls Down - Aaron Richards.flac") {
        if let Ok(tagged_file) = probe.read() {
            let properties = tagged_file.properties();
            println!("Duration: {:?}", properties.duration());
            println!("Sample rate: {:?}", properties.sample_rate());
            println!("Bitrate: {:?}", properties.audio_bitrate());

            if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                println!("Title: {:?}", tag.title());
                println!("Artist: {:?}", tag.artist());
                println!("Album: {:?}", tag.album());
            }
        }
    }
}
