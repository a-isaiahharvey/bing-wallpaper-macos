pub mod region {
    use std::collections::HashMap;

    use crate::hashmap;

    lazy_static::lazy_static! {
        pub static ref ALL: HashMap<&'static str, &'static str> = hashmap! {
            "Argentina"=> "AR",
            "Australia"=> "AU",
            "Austria"=> "AT",
            "Belgium"=> "BE",
            "Brazil"=> "BR",
            "Canada"=> "CA",
            "Chile"=> "CL",
            "Denmark"=> "DK",
            "Finland"=> "FI",
            "France"=> "FR",
            "Germany"=> "DE",
            "Hong Kong SAR"=> "HK",
            "India"=> "IN",
            "Indonesia"=> "ID",
            "Italy"=> "IT",
            "Japan"=> "JP",
            "Korea"=> "KR",
            "Malaysia"=> "MY",
            "Mexico"=> "MX",
            "Netherlands"=> "NL",
            "New Zealand"=> "NZ",
            "Norway"=> "NO",
            "China"=> "CN",
            "Poland"=> "PL",
            "Portugal"=> "PT",
            "Philippines"=> "PH",
            "Russia"=> "RU",
            "Saudi Arabia"=> "SA",
            "South Africa"=> "ZA",
            "Spain"=> "ES",
            "Sweden"=> "SE",
            "Switzerland"=> "CH",
            "Taiwan"=> "TW",
            "Turkey"=> "TR",
            "United Kingdom"=> "GB",
            "United States"=> "US",
        };
    }
}

pub mod shared {
    use std::collections::HashMap;

    use icrate::{
        objc2::rc::{autoreleasepool, Id},
        Foundation::{NSHomeDirectory, NSString, NSUserDefaults},
    };

    use crate::hashmap;

    pub enum Key {}

    impl Key {
        pub const WILL_LAUNCH_ON_SYSTEM_STARTUP: &'static str = "WillLaunchOnSystemStartup";
        pub const WILL_DISPLAY_ICON_IN_DOCK: &'static str = "WillDisplayIconInDock";
        pub const WILL_AUTO_DOWNLOAD_NEW_IMAGES: &'static str = "WillAutoDownloadNewImages";
        pub const WILL_AUTO_CHANGE_WALLPAPER: &'static str = "WillAutoChangeWallpaper";
        pub const DOWNLOADED_IMAGES_STORAGE_PATH: &'static str = "DownloadedImagesStoragePath";
        pub const CURRENT_SELECTED_BING_REGION: &'static str = "CurrentSelectedBingRegion";
        pub const CURRENT_SELECTED_IMAGE_DATE: &'static str = "CurrentSelectedImageDate";
    }

    lazy_static::lazy_static! {
        pub static ref DEFAULTS: HashMap<&'static str, String> = {
            hashmap! {
                Key::DOWNLOADED_IMAGES_STORAGE_PATH => format!("{}/Pictures/Bing Wallpaper",unsafe{ NSHomeDirectory().as_ref()}),
                Key::CURRENT_SELECTED_BING_REGION => String::new(),
            }
        };
    }

    pub fn bool(key: &str) -> bool {
        let key = NSString::from_str(key);
        unsafe { NSUserDefaults::standardUserDefaults().boolForKey(&key) }
    }

    pub fn bool_as_int(key: &str) -> i32 {
        if bool(key) {
            1
        } else {
            0
        }
    }

    pub fn set(value: bool, default_name: &NSString) {
        unsafe { NSUserDefaults::standardUserDefaults().setBool_forKey(value, default_name) }
    }

    pub fn string(key: &str) -> Option<Id<NSString>> {
        let key = NSString::from_str(key);

        unsafe {
            match NSUserDefaults::standardUserDefaults().stringForKey(&key) {
                Some(value) => Some(value),
                None => {
                    let string = NSString::from_str(
                        match autoreleasepool(|pool| DEFAULTS.get(key.as_str(pool))) {
                            Some(value) => value,
                            None => return None,
                        },
                    );

                    Some(string)
                }
            }
        }
    }

    pub fn set_with_string(string: &NSString, default_name: &str) {
        unsafe {
            let default_name = NSString::from_str(default_name);
            NSUserDefaults::standardUserDefaults().setObject_forKey(Some(string), &default_name)
        }
    }

    pub fn clear() {
        unsafe {
            for key in NSUserDefaults::standardUserDefaults()
                .dictionaryRepresentation()
                .iter_keys()
            {
                NSUserDefaults::standardUserDefaults().removeObjectForKey(key)
            }
        }
    }
}
