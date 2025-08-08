use geo::prelude::Distance;
use geo::{Haversine, Point, point};
use gpx::Gpx;

/// Sum the length of all track segments in a GPX.
pub fn gpx_total_distance(gpx: &Gpx) -> f64 {
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

/// Returns the name of the first track in a GPX file, if present.
pub fn gpx_track_name(gpx: &Gpx) -> Option<&str> {
    gpx.tracks.get(0)?.name.as_deref()
}

/// Returns the total elevation gain from a GPX file.
pub fn gpx_elevation_gain(gpx: &Gpx) -> f64 {
    let mut gain = 0.0;
    for track in &gpx.tracks {
        for segment in &track.segments {
            let mut last_elev: Option<f64> = None;
            for point in &segment.points {
                if let Some(elev) = point.elevation {
                    if let Some(prev_elev) = last_elev {
                        let diff = elev - prev_elev;
                        if diff > 0.0 {
                            gain += diff;
                        }
                    }
                    last_elev = Some(elev);
                }
            }
        }
    }

    gain
}
