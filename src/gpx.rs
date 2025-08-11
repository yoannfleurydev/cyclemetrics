use chrono::{DateTime, FixedOffset};
use geo::prelude::Distance;
use geo::{Haversine, point};
use gpx::{Gpx, Time};
use time::OffsetDateTime;

/// Sum the length of all track segments in a GPX.
pub fn gpx_total_distance(gpx: &Gpx) -> f64 {
    gpx.tracks
        .iter()
        .flat_map(|track| track.segments.iter())
        .flat_map(|segment| segment.points.windows(2)) // Taking two points
        .map(|window| {
            let (p1, p2) = (&window[0], &window[1]);
            let (lat1, lon1) = (p1.point().y(), p1.point().x());
            let (lat2, lon2) = (p2.point().y(), p2.point().x());
            let pt1 = point!(x: lon1, y: lat1);
            let pt2 = point!(x: lon2, y: lat2);
            Haversine.distance(pt1, pt2)
        })
        .sum()
}

/// Returns the name of the first track in a GPX file, if present.
pub fn gpx_track_name(gpx: &Gpx) -> Option<&str> {
    gpx.tracks.get(0)?.name.as_deref()
}

/// Returns the total elevation gain from a GPX file.
pub fn gpx_elevation_gain(gpx: &Gpx) -> f64 {
    gpx.tracks
        .iter()
        .flat_map(|track| track.segments.iter())
        .flat_map(|segment| segment.points.windows(2))
        .map(|window| {
            let (p1, p2) = (&window[0], &window[1]);

            match (p1.elevation, p2.elevation) {
                (Some(e1), Some(e2)) => {
                    let diff = e2 - e1;
                    if diff > 0.0 { diff } else { 0.0 }
                }
                _ => 0.0,
            }
        })
        .sum()
}

/// Returns the start and end date of the GPX file, if available.
pub fn gpx_start_end_date(gpx: &Gpx) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
    let times: Vec<DateTime<FixedOffset>> = gpx
        .tracks
        .iter()
        .flat_map(|track| track.segments.iter())
        .flat_map(|segment| segment.points.iter())
        .filter_map(|point| point.time.map(gpx_to_chrono))
        .collect();

    if times.is_empty() {
        None
    } else {
        let start = *times.iter().min()?;
        let end = *times.iter().max()?;

        Some((start, end))
    }
}

fn gpx_to_chrono(gpx_time: Time) -> DateTime<FixedOffset> {
    let offset_date_time: OffsetDateTime = gpx_time.into();
    let datetime_from_timestamp = DateTime::from_timestamp(
        offset_date_time.unix_timestamp(),
        offset_date_time.nanosecond(),
    )
    .unwrap();
    let offset = FixedOffset::east_opt(offset_date_time.offset().whole_seconds()).unwrap();

    datetime_from_timestamp.with_timezone(&offset.into())
}
