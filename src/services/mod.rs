//! Native desktop services, separate from the AI command planner.
pub mod connectivity;
pub mod notifications;
pub mod power;

pub mod capture;
pub mod clipboard;
pub mod credentials;
pub mod idle;
pub mod locking;
pub mod polkit;
pub mod portals;
pub mod settings;
mod tools;

#[cfg(all(test, target_os = "linux"))]
mod test_bus;
