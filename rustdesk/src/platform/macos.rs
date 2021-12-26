// https://developer.apple.com/documentation/appkit/nscursor
// https://github.com/servo/core-foundation-rs
// https://github.com/rust-windowing/winit

use super::{CursorData, ResultType};
use cocoa::{
    base::{id, nil, BOOL, NO, YES},
    foundation::{NSDictionary, NSPoint, NSSize, NSString},
};
use core_foundation::{
    array::{CFArrayGetCount, CFArrayGetValueAtIndex},
    dictionary::CFDictionaryRef,
    string::CFStringRef,
};
use core_graphics::{
    display::{kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo},
    window::{kCGWindowName, kCGWindowOwnerPID},
};
use hbb_common::{allow_err, bail, log};
use objc::{class, msg_send, sel, sel_impl};
use scrap::{libc::c_void, quartz::ffi::*};

static mut LATEST_SEED: i32 = 0;

extern "C" {
    fn CGSCurrentCursorSeed() -> i32;
    fn CGEventCreate(r: *const c_void) -> *const c_void;
    fn CGEventGetLocation(e: *const c_void) -> CGPoint;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> BOOL;
}

pub fn is_process_trusted(prompt: bool) -> bool {
    unsafe {
        let value = if prompt { YES } else { NO };
        let value: id = msg_send![class!(NSNumber), numberWithBool: value];
        let options = NSDictionary::dictionaryWithObject_forKey_(
            nil,
            value,
            kAXTrustedCheckOptionPrompt as _,
        );
        AXIsProcessTrustedWithOptions(options as _) == YES
    }
}

// macOS >= 10.15
// https://stackoverflow.com/questions/56597221/detecting-screen-recording-settings-on-macos-catalina/
// remove just one app from all the permissions: tccutil reset All com.carriez.rustdesk
pub fn is_can_screen_recording(prompt: bool) -> bool {
    let mut can_record_screen: bool = false;
    unsafe {
        let our_pid: i32 = std::process::id() as _;
        let our_pid: id = msg_send![class!(NSNumber), numberWithInteger: our_pid];
        let window_list =
            CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID);
        let n = CFArrayGetCount(window_list);
        let dock = NSString::alloc(nil).init_str("Dock");
        for i in 0..n {
            let w: id = CFArrayGetValueAtIndex(window_list, i) as _;
            let name: id = msg_send![w, valueForKey: kCGWindowName as id];
            if name.is_null() {
                continue;
            }
            let pid: id = msg_send![w, valueForKey: kCGWindowOwnerPID as id];
            let is_me: BOOL = msg_send![pid, isEqual: our_pid];
            if is_me == YES {
                continue;
            }
            let pid: i32 = msg_send![pid, intValue];
            let p: id = msg_send![
                class!(NSRunningApplication),
                runningApplicationWithProcessIdentifier: pid
            ];
            if p.is_null() {
                // ignore processes we don't have access to, such as WindowServer, which manages the windows named "Menubar" and "Backstop Menubar"
                continue;
            }
            let url: id = msg_send![p, executableURL];
            let exe_name: id = msg_send![url, lastPathComponent];
            if exe_name.is_null() {
                continue;
            }
            let is_dock: BOOL = msg_send![exe_name, isEqual: dock];
            if is_dock == YES {
                // ignore the Dock, which provides the desktop picture
                continue;
            }
            can_record_screen = true;
            break;
        }
    }
    if !can_record_screen && prompt {
        use scrap::{Capturer, Display};
        if let Ok(d) = Display::primary() {
            Capturer::new(d, true).ok();
        }
    }
    can_record_screen
}

pub fn get_cursor_pos() -> Option<(i32, i32)> {
    unsafe {
        let e = CGEventCreate(0 as _);
        let point = CGEventGetLocation(e);
        CFRelease(e);
        Some((point.x as _, point.y as _))
    }
    /*
    let mut pt: NSPoint = unsafe { msg_send![class!(NSEvent), mouseLocation] };
    let screen: id = unsafe { msg_send![class!(NSScreen), currentScreenForMouseLocation] };
    let frame: NSRect = unsafe { msg_send![screen, frame] };
    pt.x -= frame.origin.x;
    pt.y -= frame.origin.y;
    Some((pt.x as _, pt.y as _))
    */
}

pub fn get_cursor() -> ResultType<Option<u64>> {
    unsafe {
        let seed = CGSCurrentCursorSeed();
        if seed == LATEST_SEED {
            return Ok(None);
        }
        LATEST_SEED = seed;
    }
    let c = get_cursor_id()?;
    Ok(Some(c.1))
}

pub fn reset_input_cache() {
    unsafe {
        LATEST_SEED = 0;
    }
}

fn get_cursor_id() -> ResultType<(id, u64)> {
    unsafe {
        let c: id = msg_send![class!(NSCursor), currentSystemCursor];
        if c == nil {
            bail!("Failed to call [NSCursor currentSystemCursor]");
        }
        let hotspot: NSPoint = msg_send![c, hotSpot];
        let img: id = msg_send![c, image];
        if img == nil {
            bail!("Failed to call [NSCursor image]");
        }
        let size: NSSize = msg_send![img, size];
        let tif: id = msg_send![img, TIFFRepresentation];
        if tif == nil {
            bail!("Failed to call [NSImage TIFFRepresentation]");
        }
        let rep: id = msg_send![class!(NSBitmapImageRep), imageRepWithData: tif];
        if rep == nil {
            bail!("Failed to call [NSBitmapImageRep imageRepWithData]");
        }
        let rep_size: NSSize = msg_send![rep, size];
        let mut hcursor =
            size.width + size.height + hotspot.x + hotspot.y + rep_size.width + rep_size.height;
        let x = (rep_size.width * hotspot.x / size.width) as usize;
        let y = (rep_size.height * hotspot.y / size.height) as usize;
        for i in 0..2 {
            let mut x2 = x + i;
            if x2 >= rep_size.width as usize {
                x2 = rep_size.width as usize - 1;
            }
            let mut y2 = y + i;
            if y2 >= rep_size.height as usize {
                y2 = rep_size.height as usize - 1;
            }
            let color: id = msg_send![rep, colorAtX:x2 y:y2];
            if color != nil {
                let r: f64 = msg_send![color, redComponent];
                let g: f64 = msg_send![color, greenComponent];
                let b: f64 = msg_send![color, blueComponent];
                let a: f64 = msg_send![color, alphaComponent];
                hcursor += (r + g + b + a) * (255 << i) as f64;
            }
        }
        Ok((c, hcursor as _))
    }
}

// https://github.com/stweil/OSXvnc/blob/master/OSXvnc-server/mousecursor.c
pub fn get_cursor_data(hcursor: u64) -> ResultType<CursorData> {
    unsafe {
        let (c, hcursor2) = get_cursor_id()?;
        if hcursor != hcursor2 {
            bail!("cursor changed");
        }
        let hotspot: NSPoint = msg_send![c, hotSpot];
        let img: id = msg_send![c, image];
        let size: NSSize = msg_send![img, size];
        let reps: id = msg_send![img, representations];
        if reps == nil {
            bail!("Failed to call [NSImage representations]");
        }
        let nreps: usize = msg_send![reps, count];
        if nreps == 0 {
            bail!("Get empty [NSImage representations]");
        }
        let rep: id = msg_send![reps, objectAtIndex: 0];
        /*
        let n: id = msg_send![class!(NSNumber), numberWithFloat:1.0];
        let props: id = msg_send![class!(NSDictionary), dictionaryWithObject:n forKey:NSString::alloc(nil).init_str("NSImageCompressionFactor")];
        let image_data: id = msg_send![rep, representationUsingType:2 properties:props];
        let () = msg_send![image_data, writeToFile:NSString::alloc(nil).init_str("cursor.jpg") atomically:0];
        */
        let mut colors: Vec<u8> = Vec::new();
        colors.reserve((size.height * size.width) as usize * 4);
        // TIFF is rgb colrspace, no need to convert
        // let cs: id = msg_send![class!(NSColorSpace), sRGBColorSpace];
        for y in 0..(size.height as _) {
            for x in 0..(size.width as _) {
                let color: id = msg_send![rep, colorAtX:x y:y];
                // let color: id = msg_send![color, colorUsingColorSpace: cs];
                if color == nil {
                    continue;
                }
                let r: f64 = msg_send![color, redComponent];
                let g: f64 = msg_send![color, greenComponent];
                let b: f64 = msg_send![color, blueComponent];
                let a: f64 = msg_send![color, alphaComponent];
                colors.push((r * 255.) as _);
                colors.push((g * 255.) as _);
                colors.push((b * 255.) as _);
                colors.push((a * 255.) as _);
            }
        }
        Ok(CursorData {
            id: hcursor,
            colors,
            hotx: hotspot.x as _,
            hoty: hotspot.y as _,
            width: size.width as _,
            height: size.height as _,
            ..Default::default()
        })
    }
}

fn get_active_user(t: &str) -> String {
    if let Ok(output) = std::process::Command::new("ls")
        .args(vec![t, "/dev/console"])
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Some(n) = line.split_whitespace().nth(2) {
                return n.to_owned();
            }
        }
    }
    "".to_owned()
}

pub fn get_active_username() -> String {
    get_active_user("-l")
}

pub fn get_active_userid() -> String {
    get_active_user("-n")
}

pub fn is_prelogin() -> bool {
    get_active_userid() == "0"
}

pub fn is_root() -> bool {
    crate::username() == "root"
}

pub fn run_as_user(arg: &str) -> ResultType<Option<std::process::Child>> {
    let uid = get_active_userid();
    let cmd = std::env::current_exe()?;
    let task = std::process::Command::new("launchctl")
        .args(vec!["asuser", &uid, cmd.to_str().unwrap_or(""), arg])
        .spawn()?;
    Ok(Some(task))
}

pub fn lock_screen() {
    std::process::Command::new(
        "/System/Library/CoreServices/Menu Extras/User.menu/Contents/Resources/CGSession",
    )
    .arg("-suspend")
    .output()
    .ok();
}

pub fn start_os_service() {
    let mut server: Option<std::process::Child> = None;
    let mut uid = "".to_owned();
    loop {
        let tmp = get_active_userid();
        let mut start_new = false;
        if tmp != uid && !tmp.is_empty() {
            uid = tmp;
            log::info!("active uid: {}", uid);
            if let Some(ps) = server.as_mut() {
                allow_err!(ps.kill());
            }
        }
        if let Some(ps) = server.as_mut() {
            match ps.try_wait() {
                Ok(Some(_)) => {
                    server = None;
                    start_new = true;
                }
                _ => {}
            }
        } else {
            start_new = true;
        }
        if start_new {
            match crate::run_me(vec!["--server"]) {
                Ok(ps) => server = Some(ps),
                Err(err) => {
                    log::error!("Failed to start server: {}", err);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(super::SERVICE_INTERVAL));
    }
}

pub fn toggle_privacy_mode(_v: bool) {
    // https://unix.stackexchange.com/questions/17115/disable-keyboard-mouse-temporarily
}

pub fn block_input(_v: bool) {
    //
}

pub fn is_installed() -> bool {
    true
}
