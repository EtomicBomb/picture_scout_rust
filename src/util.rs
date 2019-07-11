pub fn map(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    // https://www.arduino.cc/reference/en/language/functions/math/map/
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}


/// Returns a value representing how square a rectangle is. If this function returns 1, then we have a square.
pub fn squareness(base: usize, height: usize) -> f64 {
    let (l, w) = if base > height { (base, height) } else { (height, base) };

    // l is longest side, w is shortest
    l as f64 / w as f64
}