//! macOS: the apps Launch Services offers for web links (NSWorkspace's
//! `URLsForApplicationsToOpenURL:` / `URLForApplicationToOpenURL:`, macOS 12+, read-only — NSWorkspace
//! may be used from any thread), and `open -b <bundle id> <url>` to open one.

use std::io;

use objc2::msg_send;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::NSString;
use url::Url;

use crate::browsers::parse::BUNDLE_ID;
use crate::browsers::{Browser, Browsers, Launch};
use crate::platform::browsers::spawn_detached;

/// Any https URL: Launch Services answers for the scheme.
const PROBE: &str = "https://example.com/";

fn ns_url(text: &str) -> Option<Retained<AnyObject>> {
    let class = AnyClass::get(c"NSURL")?;
    let text = NSString::from_str(text);
    // SAFETY: +[NSURL URLWithString:] takes an NSString and returns an NSURL or nil.
    unsafe { msg_send![class, URLWithString: &*text] }
}

/// An app's bundle id and the name the Finder shows, from its URL.
fn app(url: &AnyObject) -> Option<(String, String)> {
    let bundle_class = AnyClass::get(c"NSBundle")?;
    let files_class = AnyClass::get(c"NSFileManager")?;
    // SAFETY: +bundleWithURL: returns an NSBundle or nil; -bundleIdentifier an NSString or nil;
    // -[NSURL path] an NSString or nil; -displayNameAtPath: an NSString.
    unsafe {
        let bundle: Option<Retained<AnyObject>> = msg_send![bundle_class, bundleWithURL: url];
        let id: Option<Retained<NSString>> = msg_send![&*bundle?, bundleIdentifier];
        let id = id?.to_string();
        let path: Option<Retained<NSString>> = msg_send![url, path];
        let files: Option<Retained<AnyObject>> = msg_send![files_class, defaultManager];
        let name = match (files, path) {
            (Some(files), Some(path)) => {
                let name: Option<Retained<NSString>> =
                    msg_send![&*files, displayNameAtPath: &*path];
                name.map(|n| n.to_string())
            }
            _ => None,
        };
        let name = name.unwrap_or_else(|| id.clone());
        Some((id, name.trim_end_matches(".app").to_owned()))
    }
}

pub fn read() -> Browsers {
    let mut browsers = Browsers {
        self_ids: vec![BUNDLE_ID.to_owned()],
        ..Browsers::default()
    };
    autoreleasepool(|_| {
        let (Some(class), Some(probe)) = (AnyClass::get(c"NSWorkspace"), ns_url(PROBE)) else {
            return;
        };
        // SAFETY: +sharedWorkspace returns the shared NSWorkspace; -URLsForApplicationsToOpenURL:
        // (macOS 12+) an NSArray of NSURLs; -URLForApplicationToOpenURL: an NSURL or nil.
        unsafe {
            let workspace: Option<Retained<AnyObject>> = msg_send![class, sharedWorkspace];
            let Some(workspace) = workspace else {
                return;
            };
            let apps: Option<Retained<AnyObject>> =
                msg_send![&*workspace, URLsForApplicationsToOpenURL: &*probe];
            if let Some(apps) = apps {
                let count: usize = msg_send![&*apps, count];
                for index in 0..count {
                    let url: Retained<AnyObject> = msg_send![&*apps, objectAtIndex: index];
                    if let Some((id, name)) = app(&url)
                        && !browsers
                            .installed
                            .iter()
                            .any(|b| b.id.eq_ignore_ascii_case(&id))
                    {
                        browsers.installed.push(Browser {
                            id,
                            name,
                            launch: Launch::Bundle,
                        });
                    }
                }
            }
            let default: Option<Retained<AnyObject>> =
                msg_send![&*workspace, URLForApplicationToOpenURL: &*probe];
            browsers.os_default = default.and_then(|url| app(&url)).map(|(id, _)| id);
        }
    });
    browsers
}

pub fn open_with(browser: &Browser, url: &Url) -> io::Result<()> {
    spawn_detached("/usr/bin/open", &["-b", browser.id.as_str(), url.as_str()])
}

/// System Settings' "Default web browser": Desktop & Dock on macOS 13+, General on macOS 12.
pub fn open_default_apps(
    open_uri: impl FnOnce(&'static str) -> io::Result<()>,
) -> io::Result<Option<&'static str>> {
    let uri = if super::os_major() >= 13 {
        "x-apple.systempreferences:com.apple.Desktop-Settings.extension"
    } else {
        "x-apple.systempreferences:com.apple.preference.general"
    };
    open_uri(uri).map(|()| Some(uri))
}

pub fn can_open_default_apps() -> bool {
    true
}
