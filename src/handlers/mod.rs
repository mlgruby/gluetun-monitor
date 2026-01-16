//! Handlers Module
//!
//! HTTP request handlers for the API endpoints.
//! Exports the status and check handlers for use in the main router.

mod check;
mod status;

pub use check::check_handler;
pub use status::status_handler;
