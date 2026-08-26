
pub mod spec;

#[cfg(test)]
mod intent_graph {
    //! This crate's instance of the graph checks (README §4.2).
    lid_rs::intent_graph!();
}
