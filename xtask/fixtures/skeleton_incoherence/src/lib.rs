//! Check 4 fixture: a layer-0 skeleton whose signatures do not compose —
//! `parse` yields a `Result<u32, _>` that `double` cannot accept.

/// Parses a count from text.
pub fn parse(s: &str) -> Result<u32, String> {
    let _ = s;
    todo!()
}

/// Doubles a count.
pub fn double(v: u64) -> u64 {
    let _ = v;
    todo!()
}

/// Composes the two — incoherently.
pub fn run(s: &str) -> u64 {
    double(parse(s))
}
