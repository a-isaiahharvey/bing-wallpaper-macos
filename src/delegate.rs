use icrate::{
    ns_string,
    objc2::{
        class,
        declare::{Ivar, IvarDrop},
        declare_class, msg_send_id,
        rc::{Id, Owned},
        runtime::Object,
        sel, ClassType,
    },
    AppKit::{
        NSApplicationDelegate, NSImage, NSPopover, NSPopoverBehaviorTransient, NSStatusBar,
        NSStatusItem, NSVariableStatusItemLength,
    },
    Foundation::{NSBundle, NSNotification, NSObject, NSObjectProtocol, NSRectEdgeMinY},
};

use crate::view_controller::ViewController;

declare_class! {
    pub struct AppDelegate {
        popover: IvarDrop<Id<NSPopover>, "_popover">,
        status_bar_item: IvarDrop<Id<NSStatusItem>, "_status_bar_item">,
    }

    mod ivar;

    unsafe impl ClassType for AppDelegate {
        type Super = NSObject;
        const NAME: &'static str = "AppDelegate";
    }

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[method(applicationDidFinishLaunching:)]
        fn application_did_finish_launching(&mut self, _: &NSNotification) {
            unsafe {
                self.popover.setBehavior(NSPopoverBehaviorTransient);

                let view_controller = ViewController::new();
                view_controller.viewDidLoad();

                self.popover.setContentViewController(Some(&view_controller));

                Ivar::write(&mut self.status_bar_item, NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength));

                let bing_image_url = NSBundle::mainBundle().URLForResource_withExtension_subdirectory(Some(ns_string!("icon_16x16@2x")), Some(ns_string!("png")), Some(ns_string!("/images/Bing"))).unwrap();
                let bing_image = NSImage::initWithContentsOfURL(NSImage::alloc(),&bing_image_url).unwrap();


                if let Some(button) = self.status_bar_item.button() {
                    button.setImage(Some(&bing_image));
                    button.setAction(Some(sel!(togglePopover:)))
                }
            }
        }
    }

    unsafe impl AppDelegate {
        #[method(togglePopover:)]
        fn toggle_popover(&self, sender: Option<&Object>) {
            unsafe {
                if let Some(button) = self.status_bar_item.button() {
                    if self.popover.isShown() {
                        self.popover.performClose(sender)
                    } else {
                        self.popover.showRelativeToRect_ofView_preferredEdge(button.bounds(), &button, NSRectEdgeMinY);

                        if let Some(view_controller) =  self.popover.contentViewController() {
                            if let Some(window) = view_controller.view(). window() {
                                window.makeKeyAndOrderFront(sender);
                            }
                        }
                    }
                }
            }
        }
    }
}

unsafe impl NSObjectProtocol for AppDelegate {}

impl AppDelegate {
    pub fn new() -> Id<Self, Owned> {
        let mut object: Id<Self, Owned> = unsafe { msg_send_id![Self::class(), new] };
        let popover: Id<NSPopover> = unsafe { msg_send_id![class!(NSPopover), new] };

        Ivar::write(&mut object.popover, popover);

        object
    }
}
