use num_traits::{Float, FromPrimitive};
use indicatif::{ProgressBar, ProgressStyle};


/// Calculate the haversine distance between 2 points, each containing latitude (first) and longitude (second) in degree
pub fn haversine<T>(p1: (T, T), p2: (T, T), r: T) -> T
where 
    T: Float + FromPrimitive,
{
    let (lat1, lon1) = p1;
    let (lat2, lon2) = p2;

    // convert to radians
    let [lat1, lat2, lon1, lon2] = [lat1, lat2, lon1, lon2].map(|var| var.to_radians());

    let two = T::from_f64(2.0).unwrap();
    let dlat = lat2-lat1;
    let dlon = lon2-lon1;
    let a = (dlat/two).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon/two).sin().powi(2);
    let c = two * a.sqrt().asin();
    let d = r * c;

    d
}

/// Struct for every return type that contains a distance for extra information
pub struct WithDistance<T> {
    pub value: T,
    pub distance: f64,
}

pub fn green_spinner() -> ProgressBar {
    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
    pb.enable_steady_tick(std::time::Duration::from_millis(75));

    pb
}
