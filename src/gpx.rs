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
