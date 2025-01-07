use std::{backtrace::Backtrace, error::Error, fmt};

use hidapi::HidError;

pub enum DsuError {
    // WindowsError(windows::core::Error, Backtrace),
    UdpError(std::io::Error, Backtrace),
    // SdlError(sdl2::IntegerOrSdlError, Backtrace),
    Simple(String, Backtrace),
    HidError(hidapi::HidError, Backtrace)
}

impl fmt::Debug for DsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // DsuError::WindowsError(err, trace) => write!(f, "Windows SDK error: {},\n Stacktrace: \n{}", err, trace),
            DsuError::UdpError(err, trace) => write!(f, "UDP error: {},\n Stacktrace: \n{}", err, trace),
            DsuError::Simple(msg, trace) => write!(f, "{},\n Stacktrace: \n{}", msg, trace),
            // DsuError::SdlError(err, trace) => write!(f, "SDL error: {},\n Stacktrace: \n{}", err, trace),
            DsuError::HidError(err, trace) => write!(f, "HID error: {},\n Stacktrace: \n{}", err, trace),
        }
    }
}

impl fmt::Display for DsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // DsuError::WindowsError(msg, trace) => write!(f, "Windows SDK error: {},\n Stacktrace: \n{}", msg, trace),
            DsuError::UdpError(err, trace) => write!(f, "UDP error: {},\n Stacktrace: \n{}", err, trace),
            DsuError::Simple(msg, trace) => write!(f, "{},\n Stacktrace: \n{}", msg, trace),
            // DsuError::SdlError(err, trace) => write!(f, "SDL error: {},\n Stacktrace: \n{}", err, trace),
            DsuError::HidError(err, trace) => write!(f, "HID error: {},\n Stacktrace: \n{}", err, trace),
        }
    }
}

impl Error for DsuError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // DsuError::WindowsError(err, _) => Some(err),
            DsuError::UdpError(err, _) => Some(err),
            _ => None,
        }
    }
}

// impl From<sdl2::IntegerOrSdlError> for DsuError {
//     fn from(err: sdl2::IntegerOrSdlError) -> DsuError {
//         DsuError::SdlError(err, Backtrace::capture())
//     }
// }

impl From<std::io::Error> for DsuError {
    fn from(err: std::io::Error) -> DsuError {
        DsuError::UdpError(err, Backtrace::capture())
    }
}

// impl From<windows::core::Error> for DsuError {
//     fn from(err: windows::core::Error) -> DsuError {
//         DsuError::WindowsError(err, Backtrace::capture())
//     }
// }

impl From<String> for DsuError {
    fn from(err: String) -> DsuError {
        DsuError::Simple(err, Backtrace::capture())
    }
}

impl From<HidError> for DsuError {
    fn from(err: HidError) -> DsuError {
        DsuError::HidError(err, Backtrace::capture())
    }
}