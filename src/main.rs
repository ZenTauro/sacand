use std::vec::Vec;
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::prelude::*;
use std::fs;

use alsa::{self, mixer::SelemChannelId};
use notify_rust::{Notification, NotificationHandle};
use directories::BaseDirs;
// use ctrlc;

fn main() {
    let bdir = BaseDirs::new().unwrap();
    let r_dir = bdir.runtime_dir().unwrap().join("sacand");

    match fs::remove_file(r_dir.clone()) {
        Ok(_) => println!("Cleaned up previous session"),
        Err(_) => (),
    };

    println!("Attempting to bind to {}", r_dir.to_str().unwrap());

    let listener = UnixListener::bind(&r_dir).unwrap();

    println!("Bound to {}", r_dir.to_str().unwrap());

    let mut notification = None;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_conn(stream, &mut notification).expect("Something went wrong");
            }
            Err(err) => {
                println!("Error: {err}");
                break;
            }
        }
    }
}

fn handle_conn(mut stream: UnixStream, notification: &mut Option<NotificationHandle>) -> std::io::Result<()> {
    println!("Handling new connection");

    let mut msg = String::new();
    stream.read_to_string(&mut msg)?;

    println!("Received {}", &msg);

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

    // TODO: There is a better way to round up to a decimal place, but
    // I don't want to add a dependency just for that and I don't feel
    // like writing the function
    let vol = format!("{pct_vol:.2}").parse::<f64>().unwrap() * 100.0;

    match notification {
        Some(handle) => (),
        None => {
            let handle = Notification::new()
                .summary(&format!("{vol}%"))
                .show()
                .unwrap();
            *notification = Some(handle);
        }
    }

    Ok(())
}
