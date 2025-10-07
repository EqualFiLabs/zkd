//! Native backend scaffold (placeholder)
// For Task 1.1 we provide a no-op type so the workspace links cleanly.

#[derive(Debug, Default)]
pub struct NativeBackend;

impl NativeBackend {
    pub fn id() -> &'static str {
        "native@0.0"
    }
}
