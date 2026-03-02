use std::ffi::{CStr, c_int};
use std::fmt::Display;
use std::ptr::NonNull;
use std::time::Duration;

mod buffer;

use chromaprint_sys as sys;

use crate::buffer::AllocSlot;

macro_rules! assert_valid_len {
    ($len:expr) => {
        match c_int::try_from($len) {
            Ok(len) => len,
            Err(_) => unreachable!("This length should be always correct")
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Algorithm {
    Test1,
    #[default]
    Test2,
    Test3,
    Test4,
    Test5,
}

impl Algorithm {
    fn into_sys(self) -> sys::ChromaprintAlgorithm {
        use sys::ChromaprintAlgorithm as A;
        match self {
            Self::Test1 => A::TEST1,
            Self::Test2 => A::TEST2,
            Self::Test3 => A::TEST3,
            Self::Test4 => A::TEST4,
            Self::Test5 => A::TEST5,
        }
    }

    fn from_sys(algo: sys::ChromaprintAlgorithm) -> Result<Self> {
        use sys::ChromaprintAlgorithm as A;
        if algo == A::TEST1 { return Ok(Self::Test1); }
        if algo == A::TEST2 { return Ok(Self::Test2); }
        if algo == A::TEST3 { return Ok(Self::Test3); }
        if algo == A::TEST4 { return Ok(Self::Test4); }
        if algo == A::TEST5 { return Ok(Self::Test5); }
        Err(ChromaprintError::UnkownAlgorithm)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    algorithm: Algorithm,
    data: Box<[u32]>,
}

impl Fingerprint {
    pub fn decode_binary(fingerprint: &[u8]) -> Result<Self> {
        Self::decode(fingerprint, false)
    }

    pub fn decode_base64(fingerprint: &str) -> Result<Self> {
        Self::decode(fingerprint.as_bytes(), true)
    }

    #[inline]
    pub fn as_raw(&self) -> &[u32] {
        &self.data
    }

    #[inline]
    pub fn into_raw(self) -> Box<[u32]> {
        self.data
    }

    #[inline]
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    pub fn hash(&self) -> Result<u32> {
        let mut hash = 0;
        let ret = unsafe {
            sys::chromaprint_hash_fingerprint(
                self.data.as_ptr(),
                assert_valid_len!(self.data.len()),
                &mut hash
            )
        };
        check_ret(ret)?;

        Ok(hash)
    }

    pub fn encode_base64(&self) -> Result<Box<str>> {
        let data = self.encode(true)?;
        String::from_utf8(data.into_vec())
            .map(String::into_boxed_str)
            .map_err(|_| ChromaprintError::InvalidString)
    }

    pub fn encode_binary(&self) -> Result<Box<[u8]>> {
        self.encode(false)
    }

    fn decode(fingerprint: &[u8], base64: bool) -> Result<Self> {
        let mut decoded = AllocSlot::new();
        let mut size = 0;
        let mut algorithm = sys::ChromaprintAlgorithm::default();
        let ret = unsafe {
            sys::chromaprint_decode_fingerprint(
                fingerprint.as_ptr().cast(), 
                fingerprint.len() as c_int, 
                decoded.as_ptr(), 
                &mut size,
                &mut algorithm,
                if base64 { 1 } else { 0 }
            )
        };
        check_ret(ret)?;

        let data = unsafe { decoded.into_box(size as usize)? };
        let algorithm = Algorithm::from_sys(algorithm)?;
        Ok(Self {
            data,
            algorithm
        })
    }

    fn encode(&self, base64: bool) -> Result<Box<[u8]>> {
        let mut encoded = AllocSlot::new();
        let mut size = 0;
        let ret = unsafe {
            sys::chromaprint_encode_fingerprint(
                self.data.as_ptr(), 
                self.data.len() as c_int, 
                self.algorithm.into_sys(), 
                encoded.as_ptr(), 
                &mut size, 
                if base64 { 1 } else { 0 }
            )
        };
        check_ret(ret)?;

        let len = convert_chromaprint_size(size)?;
        unsafe {
            let data = encoded.into_box(len)?;
            let data = Box::into_raw(data);
            Ok(Box::from_raw(data as *mut _))
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChromaprintError {
    /// An error that happend inside the chromaprint library
    Chromaprint,
    /// Chormaprint returned an unkown algorithm number
    UnkownAlgorithm,
    /// Chromaprint returned a non UTF-8 string
    InvalidString,
    /// Chromaprint returned a null pointer
    InvalidBuffer,
    /// Chromaprint returned an invalid size
    InvalidSize,
    /// Chromaprint accepts buffers with a size of at most c_int::MAX
    InputTooLong,
}

impl Display for ChromaprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Chromaprint => write!(f, "The operation failed"),
            Self::UnkownAlgorithm => write!(f, "Got an unkown algorithm"),
            Self::InvalidString => write!(f, "Got an invalid string"),
            Self::InvalidBuffer => write!(f, "Got an invalid buffer"),
            Self::InvalidSize => write!(f, "Got an invalid size"),
            Self::InputTooLong => write!(f, "The provided buffer is too long"),
        }
    }
}

impl std::error::Error for ChromaprintError {}

type Result<T> = std::result::Result<T, ChromaprintError>;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ChromaprintOption {
    SilenceThreshold(i16),
}

impl ChromaprintOption {
    const fn name(&self) -> &'static CStr {
        match self {
            Self::SilenceThreshold(_) => unsafe {
                CStr::from_bytes_with_nul_unchecked(b"silence_threshold\0")
            }
        }
    }
}

pub struct Context {
    ctx: NonNull<sys::ChromaprintContext>,
    algorithm: Algorithm,
    sample_rate: c_int,
    channels: c_int,
}

impl Context {
    fn ctx(&self) -> *mut sys::ChromaprintContext {
        self.ctx.as_ptr()
    }

    pub fn new(
        algorithm: Algorithm,
        sample_rate: u32,
        channels: u32,
    ) -> Result<Self> {
        let ctx = unsafe {
            let new = sys::chromaprint_new(algorithm.into_sys());
            NonNull::new(new).ok_or(ChromaprintError::Chromaprint)?
        };

        let sample_rate = c_int::try_from(sample_rate)
            .expect("Sample rate is too big");
        let channels = c_int::try_from(channels)
            .expect("Channels is too big");

        let new = Self {
            ctx,
            algorithm,
            sample_rate,
            channels,
        };

        let ret = unsafe {
            sys::chromaprint_start(new.ctx(), sample_rate, channels)
        };
        check_ret(ret)?;

        Ok(new)
    }

    pub fn restart(&mut self, sample_rate: u32, channels: u32) -> Result<()> {
        let sample_rate = c_int::try_from(sample_rate)
            .expect("Sample rate is too big");
        let channels = c_int::try_from(channels)
            .expect("Channels is too big");
        let ret = unsafe {
            sys::chromaprint_start(self.ctx(), sample_rate, channels)
        };
        match check_ret(ret) {
            r @ Ok(_) => {
                self.sample_rate = sample_rate;
                self.channels = channels;
                r
            }
            r => r
        }
    }

    pub fn feed(&mut self, data: &[i16]) -> Result<()> {
        let size = c_int::try_from(data.len())
            .map_err(|_| ChromaprintError::InputTooLong)?;
        let ret = unsafe {
            sys::chromaprint_feed(self.ctx(), data.as_ptr(), size)
        };
        check_ret(ret)
    }

    pub fn finish(&mut self) -> Result<()> {
        let ret = unsafe {
            sys::chromaprint_finish(self.ctx())
        };
        check_ret(ret)
    }

    pub fn clear(&mut self) -> Result<()> {
        let ret = unsafe {
            sys::chromaprint_clear_fingerprint(self.ctx())
        };
        check_ret(ret)
    }

    pub fn get_fingerprint(&mut self) -> Result<Fingerprint> {
        let mut fingerprint = AllocSlot::new();
        let mut size = 0;
        let ret = unsafe {
            sys::chromaprint_get_raw_fingerprint(
                self.ctx(),
                fingerprint.as_ptr(),
                &mut size
            )
        };

        check_ret(ret)?;

        let data = unsafe { fingerprint.into_box(size as usize)? };

        Ok(Fingerprint {
            algorithm: self.algorithm,
            data,
        })
    }

    pub fn set_option(&mut self, option: ChromaprintOption) -> Result<()> {
        let name = option.name();
        let value = match option {
            // TODO: check that the value is positive
            ChromaprintOption::SilenceThreshold(v) => v as i32,
        };

        let ret = unsafe {
            sys::chromaprint_set_option(self.ctx(), name.as_ptr(), value)
        };

        check_ret(ret)
    }

    pub fn get_num_channels(&self) -> u32 {
        self.channels as u32
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate as u32
    }

    pub fn get_alorithm(&self) -> Algorithm {
        self.algorithm
    }

    pub fn get_delay(&self) -> Duration {
        let delay = self.get_delay_samples() as f64 / self.sample_rate as f64;
        Duration::from_secs_f64(delay)
    }

    pub fn get_delay_samples(&self) -> u32 {
        let delay = unsafe { sys::chromaprint_get_delay(self.ctx()) };
        u32::try_from(delay).unwrap()
    }

    pub fn get_duration(&self) -> Duration {
        let duration = self.get_duration_samples() as f64 / self.sample_rate as f64;
        Duration::from_secs_f64(duration)
    }

    pub fn get_duration_samples(&self) -> u32 {
        let duration = unsafe { sys::chromaprint_get_item_duration(self.ctx()) };
        u32::try_from(duration).unwrap()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { sys::chromaprint_free(self.ctx()); }
    }
}

pub fn get_version() -> &'static str {
    unsafe {
        let cstr = sys::chromaprint_get_version();
        let cstr = CStr::from_ptr(cstr);
        cstr.to_str().expect("Invalid Chromaprint version string")
    }

}

#[inline]
fn check_ret(r: std::ffi::c_int) -> Result<()> {
    if r == 1 {
        Ok(())
    } else {
        Err(ChromaprintError::Chromaprint)
    }
}

#[inline]
fn convert_chromaprint_size(size: c_int) -> Result<usize> {
    if size == 0 { return Err(ChromaprintError::InvalidSize) }
    usize::try_from(size).map_err(|_| ChromaprintError::InvalidSize)
}
