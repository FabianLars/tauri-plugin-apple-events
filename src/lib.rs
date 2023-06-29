#![cfg(target_os = "macos")]

use std::{
    ffi::CStr,
    io::{ErrorKind, Result},
    sync::Mutex,
};

use objc2::{
    class, declare_class,
    ffi::NSInteger,
    msg_send, msg_send_id,
    rc::{Id, Owned, Shared},
    runtime::{NSObject, Object},
    sel, ClassType,
};
use once_cell::sync::OnceCell;

mod apple_event;
mod consts;
use apple_event::*;
use consts::*;

type THandler = OnceCell<Mutex<Box<dyn FnMut(AppleEvent, Vec<String>) + Send + 'static>>>;

// If the Mutex turns out to be a problem, or FnMut turns out to be useless, we can remove the Mutex and turn FnMut into Fn
static HANDLER: THandler = OnceCell::new();

// Adapted from https://github.com/mrmekon/fruitbasket/blob/aad14e400d710d1d46317c0d8c55ff742bfeaadd/src/osx.rs#L848
fn parse_event(event: *mut Object) -> (AppleEvent, Option<Vec<String>>) {
    if event as u64 == 0u64 {
        return (AppleEvent::Unknown(0), None);
    }
    unsafe {
        let class: u32 = msg_send![event, eventClass];
        let id: u32 = msg_send![event, eventID];
        let event_type = AppleEvent::from(dbg!(id));
        if dbg!(dbg!(&event_type).is_unknown())
            || (class != EVENT_CLASS_CORE && class != EVENT_CLASS_INTERNET)
        {
            // The remaining msg_send! calls may panic on unknown events so we better return early.
            return (event_type, None);
        }

        let subevent: *mut Object = msg_send![event, paramDescriptorForKeyword: 0x2d2d2d2d_u32];

        if event_type == AppleEvent::kAEOpenDocuments {
            dbg!(true);
            let count: NSInteger = msg_send![subevent, numberOfItems];
            let mut docs = Vec::with_capacity(count.try_into().unwrap_or_default());

            dbg!(count);

            for i in 0..count {
                let path: *mut Object = msg_send![subevent, descriptorAtIndex: i + 1];
                let nsstring: *mut Object = msg_send![path, stringValue];
                let cstr: *const i8 = msg_send![nsstring, UTF8String];

                if !cstr.is_null() {
                    let s = CStr::from_ptr(cstr).to_string_lossy().to_string();
                    docs.push(dbg!(s));
                } else {
                    continue;
                }
            }

            dbg!(&docs);

            (event_type, if docs.is_empty() { None } else { Some(docs) })
        } else {
            dbg!(false);
            let nsstring: *mut Object = msg_send![subevent, stringValue];
            let cstr: *const i8 = msg_send![nsstring, UTF8String];

            (
                event_type,
                if !cstr.is_null() {
                    Some(vec![std::ffi::CStr::from_ptr(cstr)
                        .to_string_lossy()
                        .to_string()])
                } else {
                    None
                },
            )
        }
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

        /* #[method(openDocuments:withReplyEvent:)]
        fn handle_docs(&self, event: *mut Object, _reply: *const Object) {
            println!("handle_docs");
            let (event_type, payload) = parse_event(event);
            let mut cb = HANDLER.get().unwrap().lock().unwrap();
            cb(event_type, payload.unwrap_or_default());
        } */
    }
);

impl Handler {
    pub fn new() -> Id<Self, Owned> {
        let cls = Self::class();
        unsafe { msg_send_id![msg_send_id![cls, alloc], init] }
    }
}

// TODO: Consider merging payload into AppleEvent
pub fn listen<F: FnMut(AppleEvent, Vec<String>) + Send + 'static>(handler: F) -> Result<()> {
    match HANDLER.get() {
        Some(inner) => {
            *inner.lock().unwrap() = Box::new(handler);
        }
        None => {
            let _ = HANDLER.set(Mutex::new(Box::new(handler)));
        }
    }

    /* if HANDLER.set(Mutex::new(handler)).is_err() {
        return Err(std::io::Error::new(
            ErrorKind::AlreadyExists,
            "Handler was already set",
        ));
    } */

    unsafe {
        let event_manager: Id<Object, Shared> =
            msg_send_id![class!(NSAppleEventManager), sharedAppleEventManager];

        let handler = Handler::new();
        let handler_boxed = Box::into_raw(Box::new(handler));

        /* #[cfg(feature = "kAEGetURL")]
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_INTERNET
            andEventID:EVENT_GET_URL]; */

        #[cfg(feature = "kAEOpenDocuments")]
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_CORE
            andEventID:EVENT_OPEN_DOCUMENTS];
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_CORE
            andEventID:EVENT_PRINT_DOCUMENTS];
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_CORE
            andEventID:EVENT_OPEN_CONTENTS];
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_CORE
            andEventID:EVENT_OPEN_APPLICATION];
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_CORE
            andEventID:EVENT_REOPEN_APPLICATION];
        let _: () = msg_send![&event_manager,
            setEventHandler: &**handler_boxed
            andSelector: sel!(handleEvent:withReplyEvent:)
            forEventClass:EVENT_CLASS_CORE
            andEventID:EVENT_QUIT_APPLICATION];
    }

    Ok(())
}
