//! Optional feature modules live here.
//!
//! Keep extensions feature-gated and avoid coupling them into core paths.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionStatus {
    Planned,
    Experimental,
    Stable,
}
