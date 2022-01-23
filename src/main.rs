use std::vec::Vec;
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::prelude::*;
use std::fs;

use alsa::{self, mixer::SelemChannelId};
use notify_rust::{Notification, NotificationHandle};
use directories::BaseDirs;

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

    let vol = vol_to_pct(avg_vol, vmax);

    let new_vol = match parse_msg(&msg) {
        Msg::Nop => vol,
        Msg::Inc(inc) => {
            let new_vol = vol + (inc as f64);
            selem.set_playback_volume_all(
                pct_to_vol(new_vol, vmax)
            ).expect(&format!("Failed to increment value from {}% to {}%", vol, new_vol));
            println!("Incrementing value from {}% to {}%", vol, new_vol);
            println!("Incrementing value from {} to {}", pct_to_vol(vol, vmax), pct_to_vol(new_vol, vmax));
            new_vol
        },
        Msg::Dec(dec) => {
            let new_vol = vol - (dec as f64);
            selem.set_playback_volume_all(
                pct_to_vol(new_vol, vmax)
            ).expect(&format!("Failed to decrement value from {}% to {}%", vol, new_vol));
            println!("Decrementing value from {}% to {}%", vol, new_vol);
            println!("Incrementing value from {} to {}", pct_to_vol(vol, vmax), pct_to_vol(new_vol, vmax));
            new_vol
        }
    };

    match notification {
        Some(notification_handle) => {
            notification_handle.summary(&format!("{new_vol}%"));
            notification_handle.update();
        },
        None => {
            let handle = Notification::new()
                .summary(&format!("{new_vol}%"))
                .show()
                .unwrap();
            *notification = Some(handle);
        }
    }

    Ok(())
}

/// TODO: There is a better way to round up to a decimal place, but
/// I don't want to add a dependency just for that and I don't feel
/// like writing the function
fn pct_to_vol(val: f64, max_val: i64) -> i64 {
    let frac = val / 100.0;
    (frac.powi(3) * (max_val as f64)) as i64
}

/// TODO: There is a better way to round up to a decimal place, but
/// I don't want to add a dependency just for that and I don't feel
/// like writing the function
fn vol_to_pct(val: i64, max_val: i64) -> f64 {
    let pct_vol = ((val as f64) / (max_val as f64)).powf(1.0 / 3.0);
    format!("{pct_vol:.2}").parse::<f64>().unwrap() * 100.0
}

#[derive(Debug, Eq, PartialEq)]
enum Msg {
    Inc(u32),
    Dec(u32),
    Nop
}

fn parse_msg(msg: &str) -> Msg {
    match msg.get(0..1) {
        Some(op) => match op {
            "+" => match msg.get(1..) {
                Some(payload) => {
                    let num = payload.parse::<u32>()
                        .unwrap_or(0);
                    Msg::Inc(num)
                },
                None => Msg::Nop
            },
            "-" => match msg.get(1..) {
                Some(payload) => {
                    let num = payload.parse::<u32>()
                        .unwrap_or(0);
                    Msg::Dec(num)
                },
                None => Msg::Nop
            },
            _   => Msg::Nop,
        }
        None => Msg::Nop,
    }
}

#[cfg(test)]
mod test {
   use super::*;

    #[test]
    fn parses_ok_plus0() {
        assert_eq!(parse_msg("+0"), Msg::Inc(0));
    }
}
