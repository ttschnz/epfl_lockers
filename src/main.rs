mod model;
mod ntfy;
use model::LockerGroup;
use ntfy::send_notification;

use geo::{point, Distance, VincentyDistance};
use std::{env::var, thread::sleep, time::Duration};

#[derive(Clone, Debug)]
struct Args {
    pc_session_id: String,
    max_distance: Option<f64>,
    scanning_interval: u64,
    locker_groups: Option<Vec<String>>,
}

impl Args {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let pc_session_id = var("pc_session_id")?;
        let max_distance = var("max_distance")
            .ok()
            .and_then(|distance| distance.parse::<f64>().ok());
        let locker_groups = var("locker_groups").ok().map(|comma_separated| {
            let splitted = comma_separated.split(',');
            splitted
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
        });
        let scanning_interval = var("scanning_interval")
            .ok()
            .and_then(|interval_str| interval_str.parse::<u64>().ok())
            .unwrap_or((60 * 60) as u64); // default to 1hr

        Ok(Self {
            pc_session_id,
            max_distance,
            scanning_interval,
            locker_groups,
        })
    }
}

fn main() {
    let args = Args::from_env().unwrap();
    let target_cordinates = point!(x:46.519_704, y:6.566_954);
    loop {
        println!("Querying lockers");
        LockerGroup::request(&args.pc_session_id)
            .unwrap()
            .iter()
            .for_each(|locker_group| {
                let locker_coords =
                    point!(x:locker_group.coordinates.0, y:locker_group.coordinates.1);
                let is_in_range = args.max_distance.as_ref().is_some_and(|max_distance| {
                    let distance = locker_coords
                        .vincenty_distance(&target_cordinates)
                        .unwrap_or_else(|_e| {
                            geo::Haversine::distance(locker_coords, target_cordinates)
                        });
                    distance <= *max_distance
                });

                let name_matches = args
                    .locker_groups
                    .as_ref()
                    .is_some_and(|groups| groups.contains(&locker_group.name));

                if is_in_range || name_matches {
                    let and = is_in_range || name_matches;
                    println!(
                        "Locker {} {}{}{}. Sending notification.",
                        locker_group.name,
                        if is_in_range { "is in range" } else { "" },
                        if and { " and " } else { "" },
                        if name_matches { "name matches" } else { "" }
                    );
                    send_notification(
                        "epfl_lockers",
                        &format!("Lockers available at {}", locker_group.name),
                    )
                    .unwrap();
                }
            });

        sleep(Duration::from_secs(args.scanning_interval));
    }
}
