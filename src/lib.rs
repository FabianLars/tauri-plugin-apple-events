#![cfg(target_os = "macos")]

use std::{
    io::{ErrorKind, Result},
    sync::Mutex,
};

use objc2::{
    class, declare_class, msg_send, msg_send_id,
    rc::{Id, Owned, Shared},
    runtime::{NSObject, Object},
    sel, ClassType,
};
use once_cell::sync::OnceCell;

mod apple_event;
mod consts;
use apple_event::*;
use consts::*;

type THandler = OnceCell<Mutex<Box<dyn FnMut(AppleEvent, String) + Send + 'static>>>;

// If the Mutex turns out to be a problem, or FnMut turns out to be useless, we can remove the Mutex and turn FnMut into Fn
static HANDLER: THandler = OnceCell::new();

// Adapted from https://github.com/mrmekon/fruitbasket/blob/aad14e400d710d1d46317c0d8c55ff742bfeaadd/src/osx.rs#L848
fn parse_event(event: *mut Object) -> (AppleEvent, Option<String>) {
    if event as u64 == 0u64 {
        return (AppleEvent::Unknown(0), None);
    }
    unsafe {
        let class: u32 = msg_send![event, eventClass];
        let id: u32 = msg_send![event, eventID];
        let event_type = AppleEvent::from(id);
        if event_type.is_unknown() || class != EVENT_CLASS_CORE || class != EVENT_CLASS_INTERNET {
            // The remaining msg_send! calls may panic on unknown events so we better return early.
            return (event_type, None);
        }

        let subevent: *mut Object = msg_send![event, paramDescriptorForKeyword: 0x2d2d2d2d_u32];
        let nsstring: *mut Object = msg_send![subevent, stringValue];
        let cstr: *const i8 = msg_send![nsstring, UTF8String];
        /* if !cstr.is_null() {
            Some(std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string())
        } else {
            None
        } */
        (
            event_type,
            Some(std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string()),
        )
    }
}

declare_class!(
    struct Handler;

    unsafe impl ClassType for Handler {
        type Super = NSObject;
        const NAME: &'static str = "TauriPluginAppleEventsHandler";
    }

    unsafe impl Handler {
        #[method(handleEvent:withReplyEvent:)]
        fn handle_event(&self, event: *mut Object, _reply: *const Object) {
            let (event_type, payload) = parse_event(event);
            let mut cb = HANDLER.get().unwrap().lock().unwrap();
            cb(event_type, payload.unwrap_or_default());
        }
    }
);

impl Handler {
    pub fn new() -> Id<Self, Owned> {
        let cls = Self::class();
        unsafe { msg_send_id![msg_send_id![cls, alloc], init] }
    }
}

pub fn listen<F: FnMut(AppleEvent, String) + Send + 'static>(handler: F) -> Result<()> {
    if HANDLER.set(Mutex::new(Box::new(handler))).is_err() {
        return Err(std::io::Error::new(
            ErrorKind::AlreadyExists,
            "Handler was already set",
        ));
    }

    unsafe {
        let event_manager: Id<Object, Shared> =
            msg_send_id![class!(NSAppleEventManager), sharedAppleEventManager];

        let handler = Handler::new();
        let handler_boxed = Box::into_raw(Box::new(handler));

        #[cfg(feature = "kAEGetURL")]
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_INTERNET
            andEventID:EVENT_GET_URL];
    }

    Ok(())
}
