//! Monitoring Module
//!
//! Background tasks for VPN monitoring and notifications.
//! Includes periodic notifier and change detector.

mod change_detector;
mod periodic;

pub use change_detector::start_change_detector;
pub use periodic::start_periodic_notifier;
