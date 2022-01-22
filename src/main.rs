use std::vec::Vec;
use alsa::{self, mixer::SelemChannelId};
use notify_rust::Notification;

fn main() {
    let mixer = alsa::Mixer::new("default", false).unwrap();
    let selem_id = alsa::mixer::SelemId::new("Master", 0);
    let selem = mixer.find_selem(&selem_id).unwrap();

    let (_, vmax) = selem.get_playback_volume_range();

    let volumes: Vec<i64> = SelemChannelId::all().iter()
        .filter(|chan| selem.has_playback_channel(**chan))
        .map(|&chan| selem.get_playback_volume(chan).unwrap())
        .collect();

    let sum: i64 = volumes.iter().copied().sum();
    let avg_vol = sum / (volumes.len() as i64);

    let pct_vol = ((avg_vol as f64) / (vmax as f64)).powf(1.0 / 3.0);

    let vol = format!("{pct_vol:.2}").parse::<f64>().unwrap() * 100.0;

    Notification::new()
        .summary("Volume")
        .body(&format!("{vol}%"))
        .show()
        .unwrap();
}
