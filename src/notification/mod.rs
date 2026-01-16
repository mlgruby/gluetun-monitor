//! Notification Module
//!
//! Sends notifications to ntfy.sh or compatible services.
//! Exports the send_notification function.

mod ntfy;

pub use ntfy::send_notification;
