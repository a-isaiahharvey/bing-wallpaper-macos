use icrate::{
    ns_string,
    objc2::{
        class,
        declare::{Ivar, IvarDrop, IvarEncode},
        declare_class, msg_send_id,
        rc::{Id, Owned, Shared},
        runtime::Object,
        ClassType,
    },
    AppKit::{NSScreen, NSWorkspace},
    Foundation::{
        NSArray, NSDataWritingAtomic, NSDateFormatter, NSDictionary, NSError, NSFileManager,
        NSJSONSerialization, NSMutableURLRequest, NSObject, NSString, NSURLConnection,
        NSURLRequestUseProtocolCachePolicy, NSUTF8StringEncoding, NSURL,
    },
};

declare_class! {
    pub struct PictureManager {
        net_request: IvarDrop<Id<NSMutableURLRequest, Owned>, "_net_request">,
        file_manager: IvarDrop<Id<NSFileManager, Shared>, "_file_manager">,
        past_wallpaper_range: IvarEncode<isize, "_past_wallpaper_range">,
    }

    mod ivars;

    unsafe impl ClassType for PictureManager {
        type Super = NSObject;
        const NAME: &'static str = "PictureManager";
    }
}

impl PictureManager {
    pub fn new() -> Id<Self, Owned> {
        let mut object: Id<Self, Owned> = unsafe { msg_send_id![Self::class(), new] };

        let net_request: Id<NSMutableURLRequest, Owned> =
            unsafe { msg_send_id![class!(NSMutableURLRequest), new] };

        unsafe {
            net_request.setCachePolicy(NSURLRequestUseProtocolCachePolicy);
            net_request.setTimeoutInterval(15.);
            net_request.setHTTPMethod(ns_string!("GET"));
        }

        Ivar::write(&mut object.net_request, net_request);
        Ivar::write(&mut object.file_manager, unsafe {
            NSFileManager::defaultManager()
        });
        Ivar::write(&mut object.past_wallpaper_range, 10);

        object
    }
}

impl PictureManager {
    fn build_info_path(
        working_directory: &NSString,
        on_date: &NSString,
        at_region: &NSString,
    ) -> Id<NSString, Shared> {
        if at_region.is_empty() {
            return NSString::from_str(&format!("{working_directory}/{on_date}.json"));
        }

        NSString::from_str(&format!("{working_directory}/{on_date}_{at_region}.json"))
    }

    fn build_image_path(
        working_directory: &NSString,
        on_date: &NSString,
        at_region: &NSString,
    ) -> Id<NSString, Shared> {
        if at_region.is_empty() {
            return NSString::from_str(&format!("{working_directory}/{on_date}.jpg"));
        }

        NSString::from_str(&format!("{working_directory}/{on_date}_{at_region}.jpg"))
    }

    fn check_and_create_working_directory(
        &self,
        path: &NSString,
    ) -> Result<(), Id<icrate::Foundation::NSError>> {
        unsafe {
            self.file_manager
                .createDirectoryAtPath_withIntermediateDirectories_attributes_error(
                    path, true, None,
                )
        }
    }

    fn obtain_wallpaper(
        &mut self,
        working_directory: &NSString,
        at_index: isize,
        at_region: &NSString,
    ) -> Result<(), Id<NSError>> {
        let base_url = "http://www.bing.com/HpImageArchive.aspx";

        unsafe {
            self.net_request.setURL(Some(
                NSURL::URLWithString(&NSString::from_str(&format!(
                    "{base_url}?format=js&n=1&idx={at_index}&cc={at_region}"
                )))
                .expect("Could not get URL from String")
                .as_ref(),
            ))
        }

        let reponse_data = unsafe {
            NSURLConnection::sendSynchronousRequest_returningResponse_error(&self.net_request, None)
        };

        if let Ok(data_value) = reponse_data {
            let data: Id<NSDictionary<NSString, Object>> = unsafe {
                let json = NSJSONSerialization::JSONObjectWithData_options_error(&data_value, 0)?;
                msg_send_id![&json, self]
            };

            if let Some(objects) = unsafe { data.valueForKey(ns_string!("images")) } {
                let objects: Id<NSArray<NSDictionary<NSString, NSObject>>> =
                    unsafe { msg_send_id![&objects, self] };
                if let Some(start_date_string) =
                    unsafe { objects[0].valueForKey(ns_string!("startdate")) }
                {
                    let start_date_string: Id<NSString, Shared> =
                        unsafe { msg_send_id![&start_date_string, self] };

                    let url_string: Id<NSString, Shared> = {
                        let value = unsafe { objects[0].valueForKey(ns_string!("url")).unwrap() };

                        unsafe { msg_send_id![&value, self] }
                    };

                    let formatter: Id<NSDateFormatter, Shared> =
                        unsafe { msg_send_id![class!(NSDateFormatter), new] };

                    unsafe { formatter.setDateFormat(Some(ns_string!("yyyyMMdd"))) };

                    if let Some(start_date) =
                        unsafe { formatter.dateFromString(&start_date_string) }
                    {
                        unsafe { formatter.setDateFormat(Some(ns_string!("yyyy-MM-dd"))) };
                        let date_string = unsafe { formatter.stringFromDate(&start_date) };

                        let info_path =
                            Self::build_info_path(working_directory, &date_string, at_region);
                        let image_path =
                            Self::build_image_path(working_directory, &date_string, at_region);

                        if unsafe { !self.file_manager.fileExistsAtPath(&info_path) } {
                            self.check_and_create_working_directory(working_directory)?;

                            unsafe {
                                data_value
                                    .writeToFile_options_error(&info_path, NSDataWritingAtomic)?;
                            }
                        }

                        if unsafe { !self.file_manager.fileExistsAtPath(&image_path) } {
                            self.check_and_create_working_directory(working_directory)?;

                            if unsafe {
                                url_string.containsString(ns_string!("http://"))
                                    || url_string.containsString(ns_string!("https://"))
                            } {
                                unsafe {
                                    self.net_request
                                        .setURL(Some(&NSURL::URLWithString(&url_string).unwrap()))
                                }
                            } else {
                                unsafe {
                                    self.net_request.setURL(Some(
                                        &NSURL::URLWithString(&NSString::from_str(&format!(
                                            "https://www.bing.com{url_string}"
                                        )))
                                        .unwrap(),
                                    ))
                                }
                            }

                            let image_response_data = unsafe {
                                NSURLConnection::sendSynchronousRequest_returningResponse_error(
                                    &self.net_request,
                                    None,
                                )?
                            };

                            unsafe {
                                image_response_data.writeToURL_options_error(
                                    &NSURL::fileURLWithPath(&image_path),
                                    NSDataWritingAtomic,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn fetch_wallpapers(
        &mut self,
        working_directory: &NSString,
        at_region: &NSString,
    ) -> Result<(), Id<NSError>> {
        for index in -1..*self.past_wallpaper_range {
            self.obtain_wallpaper(working_directory, index, at_region)?;
        }
        Ok(())
    }

    pub fn fetch_last_wallpaper(
        &mut self,
        working_directory: &NSString,
        at_region: &NSString,
    ) -> Result<(), Id<NSError>> {
        for index in -1..0 {
            self.obtain_wallpaper(working_directory, index, at_region)?;
        }

        Ok(())
    }

    pub fn check_wallpaper_exist(
        &self,
        working_directory: &NSString,
        on_date: &NSString,
        at_region: &NSString,
    ) -> bool {
        unsafe {
            self.file_manager.fileExistsAtPath(&Self::build_image_path(
                working_directory,
                on_date,
                at_region,
            ))
        }
    }

    pub fn get_wallpaper_info(
        working_directory: &NSString,
        on_date: &NSString,
        at_region: &NSString,
    ) -> (Id<NSString>, Id<NSString>) {
        let json_string = unsafe {
            NSString::stringWithContentsOfFile_encoding_error(
                &Self::build_info_path(working_directory, on_date, at_region),
                NSUTF8StringEncoding,
            )
            .unwrap()
        };

        let json_string: Id<NSString> = unsafe { msg_send_id![&json_string, self] };

        if let Some(json_data) = unsafe { json_string.dataUsingEncoding(NSUTF8StringEncoding) } {
            let data: Id<NSDictionary<NSString, Object>> = unsafe {
                let json =
                    NSJSONSerialization::JSONObjectWithData_options_error(&json_data, 0).unwrap();
                msg_send_id![&json, self]
            };

            if let Some(objects) = unsafe { data.valueForKey(ns_string!("images")) } {
                let objects: Id<NSArray<NSDictionary<NSString, NSObject>>> =
                    unsafe { msg_send_id![&objects, self] };

                if let Some(copyright_string) =
                    unsafe { objects[0].valueForKey(ns_string!("copyright")) }
                {
                    let copyright_string: Id<NSString> =
                        unsafe { msg_send_id![&copyright_string, self] };

                    let copyright_link_string: Id<NSString, Shared> = {
                        let value =
                            unsafe { objects[0].valueForKey(ns_string!("copyrightlink")).unwrap() };

                        unsafe { msg_send_id![&value, self] }
                    };

                    return (copyright_string, copyright_link_string);
                }
            }
        }

        (NSString::new(), NSString::new())
    }

    pub fn set_wallpaper(
        &self,
        working_directory: &NSString,
        on_date: &NSString,
        at_region: &NSString,
    ) -> Result<(), Id<NSError>> {
        if self.check_wallpaper_exist(working_directory, on_date, at_region) {
            for screen in unsafe { NSScreen::screens().into_iter() } {
                unsafe {
                    NSWorkspace::sharedWorkspace().setDesktopImageURL_forScreen_options_error(
                        &NSURL::fileURLWithPath(&Self::build_image_path(
                            working_directory,
                            on_date,
                            at_region,
                        )),
                        screen,
                        &NSDictionary::dictionary(),
                    )?
                }
            }
        }

        Ok(())
    }
}
