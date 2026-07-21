//! System-provided device location.
//!
//! macOS: CoreLocation via objc2 (delegate-free — we poll `CLLocationManager`'s
//! `location` property each frame after requesting authorization).
//! Other platforms: a stub returning `Unavailable` (Linux gpsd goes here later).
//!
//! NOTE: CoreLocation only delivers a fix when the process is a bundled `.app`
//! carrying `NSLocationWhenInUseUsageDescription`. From a bare binary, auth is
//! denied and this reports `Unavailable`, so the caller falls back to IP.

use crate::location::Coords;

/// One poll result from the system locator.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SystemFix {
    Pending,
    Ready(Coords),
    Unavailable,
}

#[cfg(target_os = "macos")]
pub struct SystemLocator {
    manager: objc2::rc::Retained<objc2_core_location::CLLocationManager>,
}

#[cfg(target_os = "macos")]
impl SystemLocator {
    pub fn start() -> Self {
        use objc2_core_location::CLLocationManager;
        // SAFETY: standard CoreLocation setup on the main thread (macroquad's
        // event loop runs the Cocoa run loop that delivers updates).
        unsafe {
            let manager = CLLocationManager::new();
            manager.requestWhenInUseAuthorization();
            manager.startUpdatingLocation();
            SystemLocator { manager }
        }
    }

    pub fn poll(&mut self) -> SystemFix {
        use objc2_core_location::CLAuthorizationStatus;
        // SAFETY: reading authorization status + last location property.
        unsafe {
            let status = self.manager.authorizationStatus();
            let authorized = status == CLAuthorizationStatus::AuthorizedWhenInUse
                || status == CLAuthorizationStatus::AuthorizedAlways;
            if status == CLAuthorizationStatus::Denied
                || status == CLAuthorizationStatus::Restricted
            {
                return SystemFix::Unavailable;
            }
            if authorized
                && let Some(loc) = self.manager.location()
            {
                let c = loc.coordinate();
                return SystemFix::Ready(Coords { lat: c.latitude, lon: c.longitude });
            }
            SystemFix::Pending
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub struct SystemLocator;

#[cfg(not(target_os = "macos"))]
impl SystemLocator {
    /// Stub: no system location provider yet (Linux gpsd goes here later).
    pub fn start() -> Self {
        SystemLocator
    }
    pub fn poll(&mut self) -> SystemFix {
        SystemFix::Unavailable
    }
}
