//! Check 7 fixture: a leaf full of branches nobody declared as claims.

/// Classifies a reading — five undeclared decisions deep.
pub fn classify(v: i64, calibrated: i64) -> &'static str {
    if v < 0 {
        if calibrated < 0 {
            if v < calibrated { "far-low" } else { "low" }
        } else {
            "negative"
        }
    } else if v > 100 {
        if calibrated > 100 {
            if v > calibrated { "far-high" } else { "high" }
        } else {
            "overflow"
        }
    } else {
        "normal"
    }
}
