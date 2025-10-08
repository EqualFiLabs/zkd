//! Trace shape derived from AIR and/or profile hints.

use crate::air::AirProgram;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceShape {
    pub rows: u32, // number of rows in the main trace
    pub cols: u32, // number of columns in the main trace
    pub const_cols: u32,
    pub periodic_cols: u32,
}

impl TraceShape {
    /// Derive a conservative TraceShape from an AIR program.
    /// If rows_hint is missing, default to 2^16 for Phase-0 demos.
    pub fn from_air(air: &AirProgram) -> Self {
        let rows = air.rows_hint.unwrap_or(1 << 16);
        Self {
            rows,
            cols: air.columns.trace_cols,
            const_cols: air.columns.const_cols,
            periodic_cols: air.columns.periodic_cols,
        }
    }
}
