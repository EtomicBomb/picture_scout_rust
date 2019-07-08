pub fn map(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    // https://www.arduino.cc/reference/en/language/functions/math/map/
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

