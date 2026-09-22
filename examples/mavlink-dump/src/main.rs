use mavlink::MavConnection;
use mavlink::error::MessageReadError;
use std::{env, sync::Arc, thread, time::Duration};

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!(
            "Usage: mavlink-dump (tcpout|tcpin|udpout|udpin|udpbcast|serial|file):(ip|dev|path):(port|baud)"
        );
        return;
    }

    // It's possible to change the mavlink dialect to be used in the connect call
    let mavconn = mavlink::connect::<mavlink::dialects::swingby::MavMessage>(&args[1]).unwrap();

    let vehicle = Arc::new(mavconn);

    thread::spawn({
        let vehicle = vehicle.clone();
        move || loop {
            let res = vehicle.send_default(&heartbeat_message());
            if res.is_ok() {
                thread::sleep(Duration::from_secs(1));
            } else {
                println!("send failed: {res:?}");
            }
        }
    });

    loop {
        match vehicle.recv() {
            Ok((_header, msg)) => {
                println!("received: {msg:?}");
            }
            Err(MessageReadError::Io(e)) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    //no messages currently available to receive -- wait a while
                    thread::sleep(Duration::from_secs(1));
                    continue;
                } else {
                    println!("recv error: {e:?}");
                    break;
                }
            }
            // messages that didn't get through due to parser errors are ignored
            _ => {}
        }
    }
}

/// Create a heartbeat message using 'swingby' dialect
pub fn heartbeat_message() -> mavlink::dialects::swingby::MavMessage {
    mavlink::dialects::swingby::MavMessage::HEARTBEAT(mavlink::dialects::swingby::HEARTBEAT_DATA {
        custom_mode: 0,
        mavtype: mavlink::dialects::swingby::MavType::MAV_TYPE_GENERIC,
        autopilot: mavlink::dialects::swingby::MavAutopilot::MAV_AUTOPILOT_GENERIC,
        base_mode: mavlink::dialects::swingby::MavModeFlag::empty(),
        system_status: mavlink::dialects::swingby::MavState::MAV_STATE_STANDBY,
        mavlink_version: 0x3,
    })
}
