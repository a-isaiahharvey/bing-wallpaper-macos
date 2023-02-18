use std::{cell::RefCell, sync::Arc};

use icrate::{
    ns_string,
    objc2::{
        class,
        declare::{Ivar, IvarDrop},
        declare_class, msg_send, msg_send_id,
        rc::{Id, Owned},
        runtime::Object,
        sel, ClassType,
    },
    AppKit::{
        NSApplication, NSApplicationActivationPolicyAccessory,
        NSApplicationActivationPolicyRegular, NSButton, NSFont, NSImage,
        NSImageScaleProportionallyDown, NSImageView, NSLineBreakByClipping, NSResponder,
        NSTextAlignmentCenter, NSTextField, NSTextView, NSView, NSViewController,
        NSWindowController, NSWorkspace,
    },
    Foundation::{
        CGPoint, CGRect, CGSize, NSBundle, NSDate, NSDateFormatter, NSError, NSObjectProtocol,
        NSPoint, NSRect, NSSize, NSString, NSTimeInterval, NSTimer, NSURL,
    },
};

use crate::{picture_manager::PictureManager, preferences};

declare_class! {
    pub struct ViewController {
        timer_task: IvarDrop<Id<NSTimer>, "_timer_task">,
        previous_date_string: IvarDrop<Id<NSString>, "_previous_date_string">,
        next_date_string: IvarDrop<Id<NSString>, "_next_date_string">,
        wallpaper_info_url_string: IvarDrop<Id<NSString>, "_wallpaper_info_url_string">,
        picture_manager: IvarDrop<Id<PictureManager, Owned>, "_picture_manager">,
        preferences_window_controller: IvarDrop<Id<NSWindowController>, "_preferences_window_controller">,

        previous_day_button: IvarDrop<Id<NSButton>, "_previous_day_button">,
        today_button: IvarDrop<Id<NSButton>, "_today_button">,
        next_day_button: IvarDrop<Id<NSButton>, "_next_day_button">,
        preferences_button: IvarDrop<Id<NSButton>, "_preferences_button">,
        quit_button: IvarDrop<Id<NSButton>, "_quit_button">,
        wallpaper_info_textview: IvarDrop<Id<NSTextView>, "_wallpaper_info_textfield">,

        bing_logo_image: IvarDrop<Id<NSImageView>, "_bing_logo_button">,

        date_textfield: IvarDrop<Id<NSTextField>, "_date_textfield">,
    }

    mod ivars;

    unsafe impl ClassType for ViewController {
        #[inherits(NSResponder)]
        type Super = NSViewController;
        const NAME: &'static str = "ViewController";
    }

    unsafe impl ViewController {
        #[method(viewDidLoad)]
        fn view_did_load(&mut self) {
            self.initialize_view();
            Self::setup_dock_icon();
            self.setup_timer_task();
            self.setup_date_textfield();
            self.setup_wallpaper_info_textview();
            self.setup_today_button();
            self.setup_quit_button();
            self.setup_bing_wallpaper_label();
            self.setup_preferences_button();
            self.setup_previous_day_button();
            self.setup_next_day_button();
            self.setup_bing_logo();

            let _ : () = unsafe {msg_send![&*self, downloadWallpapers]};

            if let Some(current_date) = preferences::shared::string(preferences::shared::Key::CURRENT_SELECTED_IMAGE_DATE) {
                let _ = self.jump_to_date(&current_date);
            } else {
                let _ = self.jump_to_today();
            }
        }

        #[method(previousDay:)]
        fn previous_day(&mut self, _: Option<&Object>) {

            let _ = self.jump_to_date(&self.previous_date_string.clone());

        }

        #[method(today:)]
        fn today(&mut self, _: Option<&Object>) {
            let _ = self.jump_to_today();
        }

        #[method(nextDay:)]
        fn next_day(&mut self, _: Option<&Object>) {
            let _ = self.jump_to_date(&self.next_date_string.clone());

        }

        #[method(wallpaperInfoButtonClicked:)]
        fn wallpaper_info_button_clicked(&self, _: Option<&Object>) {
            unsafe {
                NSWorkspace::sharedWorkspace().openURL(NSURL::URLWithString(self.wallpaper_info_url_string.as_ref().as_ref()).unwrap().as_ref());
            }
        }

        #[method(downloadWallpapers)]
        fn download_wallpapers(&mut self) {
            if let Some(working_directory) = preferences::shared::string(
                preferences::shared::Key::DOWNLOADED_IMAGES_STORAGE_PATH,
            ) {
                if let Some(region) = preferences::shared::string(
                    preferences::shared::Key::CURRENT_SELECTED_BING_REGION,
                ) {
                    let _ = self.picture_manager
                        .fetch_wallpapers(&working_directory, &region);

                }
            }

            if preferences::shared::bool(preferences::shared::Key::WILL_AUTO_CHANGE_WALLPAPER) {
                unsafe {
                    let formatter: Id<NSDateFormatter> = msg_send_id![class!(NSDateFormatter), new];

                    formatter.setDateFormat(Some(ns_string!("yyyy-MM-dd")));
                    let _ = self.jump_to_date(&formatter.stringFromDate(&NSDate::date()));
                }
            }
        }

        #[method(quitApplication:)]
        fn quit_application(&self, sender: Option<&Object>) {
            unsafe { NSApplication::sharedApplication().terminate(sender) }
        }
    }
}

unsafe impl NSObjectProtocol for ViewController {}

impl ViewController {
    pub fn new() -> Id<Self, Owned> {
        let mut object: Id<Self, Owned> = unsafe { msg_send_id![Self::class(), new] };
        Ivar::write(&mut object.picture_manager, PictureManager::new());

        Ivar::write(&mut object.previous_date_string, unsafe {
            NSString::string()
        });
        Ivar::write(&mut object.next_date_string, unsafe { NSString::string() });
        Ivar::write(&mut object.wallpaper_info_url_string, unsafe {
            NSString::string()
        });

        unsafe {
            let date_text_field = NSTextField::labelWithString(ns_string!(""));
            Ivar::write(&mut object.date_textfield, date_text_field);

            let previous_day_button: Id<NSButton> = NSButton::buttonWithImage_target_action(
                &NSImage::imageNamed(ns_string!("NSLeftFacingTriangleTemplate")).unwrap(),
                None,
                Some(sel!(previousDay:)),
            );
            Ivar::write(&mut object.previous_day_button, previous_day_button);

            let today_button: Id<NSButton> = NSButton::buttonWithTitle_target_action(
                ns_string!("Today"),
                Some(&object),
                Some(sel!(today:)),
            );
            Ivar::write(&mut object.today_button, today_button);

            let next_day_button: Id<NSButton> = NSButton::buttonWithImage_target_action(
                &NSImage::imageNamed(ns_string!("NSRightFacingTriangleTemplate")).unwrap(),
                Some(&object),
                Some(sel!(nextDay:)),
            );
            Ivar::write(&mut object.next_day_button, next_day_button);

            let preferences_button: Id<NSButton> = NSButton::buttonWithTitle_target_action(
                ns_string!("Preferences"),
                Some(&object),
                None,
            );
            Ivar::write(&mut object.preferences_button, preferences_button);

            let wallpaper_info_textview: Id<NSTextView> = NSTextView::initWithFrame(
                NSTextView::alloc(),
                CGRect {
                    origin: CGPoint { x: 28., y: 52. },
                    size: CGSize {
                        width: 164.,
                        height: 80.,
                    },
                },
            );
            Ivar::write(&mut object.wallpaper_info_textview, wallpaper_info_textview);

            let quit_button: Id<NSButton> = NSButton::buttonWithTitle_target_action(
                ns_string!("Quit"),
                Some(&object),
                Some(sel!(quitApplication:)),
            );
            Ivar::write(&mut object.quit_button, quit_button);

            let bing_image_url = NSBundle::mainBundle()
                .URLForResource_withExtension_subdirectory(
                    Some(ns_string!("icon_32x32@2x")),
                    Some(ns_string!("png")),
                    Some(ns_string!("/images/Bing")),
                )
                .unwrap();
            let bing_image =
                NSImage::initWithContentsOfURL(NSImage::alloc(), &bing_image_url).unwrap();
            let bing_logo_image_view: Id<NSImageView> =
                NSImageView::imageViewWithImage(&bing_image);
            Ivar::write(&mut object.bing_logo_image, bing_logo_image_view);
        }

        object
    }

    fn initialize_view(&self) {
        unsafe {
            self.setView(&NSView::initWithFrame(
                NSView::alloc(),
                NSRect {
                    origin: NSPoint { x: 0., y: 0. },
                    size: NSSize {
                        width: 220.,
                        height: 300.,
                    },
                },
            ))
        };
    }

    fn jump_to_date(&mut self, date: &NSString) -> Result<bool, Id<NSError>> {
        if let Some(working_directory) =
            preferences::shared::string(preferences::shared::Key::DOWNLOADED_IMAGES_STORAGE_PATH)
        {
            if let Some(region) =
                preferences::shared::string(preferences::shared::Key::CURRENT_SELECTED_BING_REGION)
            {
                if self
                    .picture_manager
                    .check_wallpaper_exist(&working_directory, date, &region)
                {
                    let info =
                        PictureManager::get_wallpaper_info(&working_directory, date, &region);

                    let mut info_string = info.0;
                    info_string = unsafe {
                        info_string.stringByReplacingOccurrencesOfString_withString(
                            ns_string!(","),
                            ns_string!("\n"),
                        )
                    };
                    info_string = unsafe {
                        info_string.stringByReplacingOccurrencesOfString_withString(
                            ns_string!("("),
                            ns_string!("\n"),
                        )
                    };
                    info_string = unsafe {
                        info_string.stringByReplacingOccurrencesOfString_withString(
                            ns_string!(")"),
                            ns_string!(""),
                        )
                    };

                    unsafe { self.wallpaper_info_textview.setString(&info_string) };
                    Ivar::write(&mut self.wallpaper_info_url_string, info.1);

                    unsafe {
                        self.wallpaper_info_textview.sizeToFit();
                    }

                    unsafe {
                        self.date_textfield.setStringValue(date);
                    }
                    preferences::shared::set_with_string(
                        date,
                        preferences::shared::Key::CURRENT_SELECTED_IMAGE_DATE,
                    );

                    self.picture_manager
                        .set_wallpaper(&working_directory, date, &region)?;

                    let search_limit = 365;
                    let formatter: Id<NSDateFormatter> =
                        unsafe { msg_send_id![class!(NSDateFormatter), new] };

                    unsafe { formatter.setDateFormat(Some(ns_string!("yyyy-MM-dd"))) };

                    if let Some(date) = unsafe { formatter.dateFromString(date) } {
                        unsafe {
                            self.previous_day_button.setEnabled(false);

                            for index in 1..search_limit {
                                let time_interval = -3600. * 24. * index as NSTimeInterval;
                                let another_day = date.dateByAddingTimeInterval(time_interval);
                                let another_day_string =
                                    formatter.stringForObjectValue(Some(&another_day)).unwrap();

                                if self.picture_manager.check_wallpaper_exist(
                                    &working_directory,
                                    &another_day_string,
                                    &region,
                                ) {
                                    Ivar::write(&mut self.previous_date_string, another_day_string);
                                    self.previous_day_button.setEnabled(true);

                                    break;
                                }
                            }
                        }
                    }

                    if let Some(date) = unsafe { formatter.dateFromString(date) } {
                        unsafe {
                            self.next_day_button.setEnabled(false);

                            for index in 1..search_limit {
                                let time_interval = 3600. * 24. * index as NSTimeInterval;
                                let another_day = date.dateByAddingTimeInterval(time_interval);
                                let another_day_string =
                                    formatter.stringForObjectValue(Some(&another_day)).unwrap();

                                if self.picture_manager.check_wallpaper_exist(
                                    &working_directory,
                                    &another_day_string,
                                    &region,
                                ) {
                                    Ivar::write(&mut self.next_date_string, another_day_string);
                                    self.next_day_button.setEnabled(true);

                                    break;
                                }
                            }
                        }
                    }

                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn jump_to_today(&mut self) -> Result<(), Id<NSError>> {
        unsafe {
            self.today_button.setEnabled(false);
            self.today_button.setTitle(ns_string!("Fetching..."));

            if let Some(working_directory) = preferences::shared::string(
                preferences::shared::Key::DOWNLOADED_IMAGES_STORAGE_PATH,
            ) {
                if let Some(current_region) = preferences::shared::string(
                    preferences::shared::Key::CURRENT_SELECTED_BING_REGION,
                ) {
                    self.picture_manager
                        .fetch_last_wallpaper(&working_directory, &current_region)?;
                }
            }

            let formatter: Id<NSDateFormatter> = msg_send_id![class!(NSDateFormatter), new];
            formatter.setDateFormat(Some(ns_string!("yyyy-MM-dd")));

            let ok = self.jump_to_date(&formatter.stringFromDate(&NSDate::date()))?;

            if !ok {
                _ = self.jump_to_date(
                    &formatter
                        .stringForObjectValue(Some(
                            NSDate::date()
                                .dateByAddingTimeInterval(-(3600. * 24.))
                                .as_ref(),
                        ))
                        .unwrap(),
                )
            }

            self.today_button.setEnabled(true);
            self.today_button.setTitle(ns_string!("Today"));

            Ok(())
        }
    }

    fn setup_dock_icon() {
        if preferences::shared::bool(preferences::shared::Key::WILL_DISPLAY_ICON_IN_DOCK) {
            unsafe {
                NSApplication::sharedApplication()
                    .setActivationPolicy(NSApplicationActivationPolicyRegular);
            }
        } else {
            unsafe {
                NSApplication::sharedApplication()
                    .setActivationPolicy(NSApplicationActivationPolicyAccessory);
            }
        }
    }

    fn setup_timer_task(&mut self) {
        let arc_self = Arc::new(RefCell::new(self));
        unsafe {
            let timer = NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                3600.,
                &arc_self.borrow(),
                sel!(downloadWallpapers),
                None,
                true,
            );

            Ivar::write(&mut arc_self.borrow_mut().timer_task, timer);
        }
    }

    fn setup_date_textfield(&mut self) {
        unsafe {
            self.date_textfield.setFrame(CGRect {
                origin: CGPoint { x: 40., y: 139. },
                size: CGSize {
                    width: 141.,
                    height: 17.,
                },
            });
            self.date_textfield.setLineBreakMode(NSLineBreakByClipping);
            self.date_textfield.setAlignment(NSTextAlignmentCenter);

            self.view().addSubview(&self.date_textfield);
        }
    }

    fn setup_wallpaper_info_textview(&mut self) {
        unsafe {
            self.wallpaper_info_textview.setFrame(CGRect {
                origin: CGPoint { x: 28., y: 52. },
                size: CGSize {
                    width: 164.,
                    height: 250.,
                },
            });

            self.wallpaper_info_textview
                .setAlignment(NSTextAlignmentCenter);
            self.wallpaper_info_textview.setDrawsBackground(false);
            self.wallpaper_info_textview.setSelectable(false);

            self.view().addSubview(&self.wallpaper_info_textview);
        }
    }

    fn setup_quit_button(&self) {
        unsafe {
            self.quit_button.setTitle(ns_string!("Quit"));
            self.quit_button.setFrame(CGRect {
                origin: CGPoint { x: 88., y: 11. },
                size: CGSize {
                    width: 50.,
                    height: 32.,
                },
            });
            self.quit_button.setAlignment(NSTextAlignmentCenter);
            self.quit_button
                .setImageScaling(NSImageScaleProportionallyDown);

            self.view().addSubview(&self.quit_button);
        };
    }

    fn setup_bing_wallpaper_label(&self) {
        unsafe {
            let text_field = NSTextField::labelWithString(ns_string!("Bing Wallpaper"));
            text_field.setFrame(CGRect {
                origin: CGPoint { x: 50., y: 229. },
                size: CGSize {
                    width: 132.,
                    height: 22.,
                },
            });
            text_field
                .as_super()
                .setFont(Some(&NSFont::systemFontOfSize(18.)));

            self.view().addSubview(&text_field);
        }
    }

    fn setup_today_button(&self) {
        unsafe {
            self.today_button.setFrame(CGRect {
                origin: CGPoint { x: 68., y: 162. },
                size: CGSize {
                    width: 85.,
                    height: 32.,
                },
            });
            self.today_button.setAlignment(NSTextAlignmentCenter);
            self.today_button
                .setImageScaling(NSImageScaleProportionallyDown);

            self.view().addSubview(&self.today_button);
        }
    }

    fn setup_preferences_button(&self) {
        unsafe {
            self.preferences_button.setFrame(CGRect {
                origin: CGPoint { x: 56., y: 188. },
                size: CGSize {
                    width: 108.,
                    height: 32.,
                },
            });
            self.preferences_button.setAlignment(NSTextAlignmentCenter);
            self.preferences_button
                .setImageScaling(NSImageScaleProportionallyDown);
            // self.view().addSubview(&self.preferences_button);
        }
    }

    fn setup_previous_day_button(&self) {
        unsafe {
            self.previous_day_button.setFrame(CGRect {
                origin: CGPoint { x: 36., y: 162. },
                size: CGSize {
                    width: 37.,
                    height: 32.,
                },
            });
            self.previous_day_button.setAlignment(NSTextAlignmentCenter);
            self.previous_day_button
                .setImageScaling(NSImageScaleProportionallyDown);
            self.view().addSubview(&self.previous_day_button);
        }
    }

    fn setup_next_day_button(&self) {
        unsafe {
            self.next_day_button.setFrame(CGRect {
                origin: CGPoint { x: 148., y: 162. },
                size: CGSize {
                    width: 37.,
                    height: 32.,
                },
            });
            self.next_day_button.setAlignment(NSTextAlignmentCenter);
            self.next_day_button
                .setImageScaling(NSImageScaleProportionallyDown);
            self.view().addSubview(&self.next_day_button)
        }
    }

    fn setup_bing_logo(&self) {
        unsafe {
            self.bing_logo_image.setFrame(CGRect {
                origin: CGPoint { x: 92., y: 250. },
                size: CGSize {
                    width: 37.,
                    height: 32.,
                },
            });
            self.bing_logo_image.setAlignment(NSTextAlignmentCenter);
            self.bing_logo_image
                .setImageScaling(NSImageScaleProportionallyDown);
            self.view().addSubview(&self.bing_logo_image)
        }
    }
}
