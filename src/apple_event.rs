use crate::consts::*;

#[non_exhaustive]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub enum AppleEvent {
    #[cfg(feature = "kAEApplicationDied")]
    kAEApplicationDied,
    #[cfg(feature = "kAEOpenApplication")]
    kAEOpenApplication,
    #[cfg(feature = "kAEOpenContents")]
    kAEOpenContents,
    #[cfg(feature = "kAEOpenDocuments")]
    kAEOpenDocuments,
    #[cfg(feature = "kAEPrintDocuments")]
    kAEPrintDocuments,
    #[cfg(feature = "kAEQuitApplication")]
    kAEQuitApplication,
    #[cfg(feature = "kAEReopenApplication")]
    kAEReopenApplication,
    #[cfg(feature = "kAEShowPreferences")]
    kAEShowPreferences,
    #[cfg(feature = "kAEGetURL")]
    kAEGetURL,
    /// Unknown event type.
    /// The stored `u32` is the event key received from the system.
    Unknown(u32),
}

impl AppleEvent {
    pub fn is_unknown(&self) -> bool {
        match *self {
            Self::Unknown(_) => true,
            _ => false,
        }
    }
}

impl From<u32> for AppleEvent {
    fn from(value: u32) -> Self {
        match value {
            EVENT_OPEN_APPLICATION => Self::kAEOpenApplication,
            EVENT_REOPEN_APPLICATION => Self::kAEReopenApplication,
            EVENT_OPEN_DOCUMENTS => Self::kAEOpenDocuments,
            EVENT_PRINT_DOCUMENTS => Self::kAEPrintDocuments,
            EVENT_OPEN_CONTENTS => Self::kAEOpenContents,
            EVENT_QUIT_APPLICATION => Self::kAEQuitApplication,
            EVENT_SHOW_PREFERENCES => Self::kAEShowPreferences,
            EVENT_APPLICATION_DIED => Self::kAEApplicationDied,
            EVENT_GET_URL => Self::kAEGetURL,
            _ => Self::Unknown(value),
        }
    }
}
