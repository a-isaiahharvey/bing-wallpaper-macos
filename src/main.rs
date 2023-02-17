use delegate::AppDelegate;
use icrate::{
    objc2::ProtocolObject,
    AppKit::{NSApplication, NSApplicationActivationPolicyAccessory},
};

pub mod delegate;
pub mod picture_manager;
pub mod preferences;
pub mod utils;
pub mod view_controller;

fn main() {
    let app = unsafe { NSApplication::sharedApplication() };

    unsafe {
        let app_delegate = AppDelegate::new();
        let app_delegate = ProtocolObject::from_ref(&*app_delegate);
        app.setDelegate(Some(app_delegate));

        app.setActivationPolicy(NSApplicationActivationPolicyAccessory);
        app.activateIgnoringOtherApps(true);
        app.run();
    }
}
