use clap::Parser;
use geo::prelude::*;
use geo::{Haversine, Point, point};
use gpx::{Gpx, read};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Compute the total track distance of one or more GPX files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths or glob patterns pointing to GPX files
    #[arg(required = true)]
    gpx_files: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Keep a running total over all files
    let mut grand_total_km: f64 = 0.0;

    // Iterate over the supplied paths / glob patterns
    for gpx_path in &args.gpx_files {
        // Resolve glob patterns if necessary
        let files = glob::glob(gpx_path.to_str().unwrap())?;
        for file_res in files {
            let file_path = file_res?;
            // Read the GPX file
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);
            let gpx: Gpx = read(reader)?;

            // Compute distance
            let distance_m = gpx_total_distance(&gpx);
            let distance_km = distance_m / 1_000.0;
            grand_total_km += distance_km;

            println!(
                "File: {:<30}  Distance: {:>8.3} km",
                file_path.display(),
                distance_km
            );
        }
    }

    println!("────────────────────────────────────");
    println!("Grand total: {:>8.3} km", grand_total_km);

    Ok(())
}

/// Sum the length of all track segments in a GPX.
fn gpx_total_distance(gpx: &Gpx) -> f64 {
    let mut total: f64 = 0.0;

    for track in &gpx.tracks {
        for segment in &track.segments {
            let mut last_point: Option<Point> = None;
            for point in &segment.points {
                let (lat, lon) = (point.point().y(), point.point().x());
                let current = point!(x: lon, y: lat);
                if let Some(prev) = last_point {
                    total += Haversine.distance(prev, current); // prev.haversine_distance(&current);
                }
                last_point = Some(current);
            }
        }
    }

    total
}
