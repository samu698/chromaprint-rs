#![no_std]

use core::ffi::{c_char, c_int, c_void};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Algorithm to use for fingerprint generation
pub struct ChromaprintAlgorithm(c_int);

impl ChromaprintAlgorithm {
    pub const TEST1: Self = Self(0);
    pub const TEST2: Self = Self(1);
    pub const TEST3: Self = Self(2);
    pub const TEST4: Self = Self(3);
    pub const TEST5: Self = Self(4);

    /// Default algorithm used
    pub const DEFAULT: Self = Self::TEST2;
}

impl Default for ChromaprintAlgorithm {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Main handle used to generate a Chromaprint fingerprint.
///
/// This type is opaque and should be accessed only through a pointer.
///
/// An instance of this buffer can be constructed with [`chromaprint_new`], then
/// the computation of a fingerprint can be started using [`chromaprint_start`],
/// audio samples can be added using [`chromaprint_feed`], finally you can 
/// finish the generation of the fingerprint using [`chromaprint_finish`] and 
/// use [`chromaprint_get_fingerprint`] to get the result.
///
/// This type must be freed using [`chromaprint_free`].
pub enum ChromaprintContext {}

#[link(name = "chromaprint")]
unsafe extern "C" {
    /// Return the version number of Chromaprint.
    pub fn chromaprint_get_version() -> *const c_char;

    /// Allocate and initialize the Chromaprint context.
    ///
    /// <div class="warning">
    /// when Chromaprint is compiled with FFTW, this function is not reentrant 
    /// and you need to call it only from one thread at a time.
    /// </div>
    pub fn chromaprint_new(algorithm: ChromaprintAlgorithm) -> *mut ChromaprintContext;

    /// Allocate and initialize the Chromaprint context.
    ///
    /// <div class="warning">
    /// when Chromaprint is compiled with FFTW, this function is not reentrant 
    /// and you need to call it only from one thread at a time.
    /// </div>
    pub fn chromaprint_free(ctx: *mut ChromaprintContext);

    /// Return the fingerprint algorithm this context is configured to use.
    pub fn chromaprint_get_algorithm(ctx: *mut ChromaprintContext) -> ChromaprintAlgorithm;

    /// Set a configuration option for the selected fingerprint algorithm.
    ///
    /// <div class="warning">
    /// Do not use this function if you are planning to use the generated 
    /// fingerprints with the acoustid service.
    /// </div>
    ///
    /// Possible options:
    /// - silence_threshold: threshold for detecting silence, 0-32767
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_set_option(
        ctx: *mut ChromaprintContext,
        name: *const c_char,
        value: c_int
    ) -> c_int;

    /// Get the number of channels that is internally used for fingerprinting.
    ///
    /// You normally don't need this. Just set the audio's actual number of 
    /// channels when calling chromaprint_start() and everything will work. 
    /// This is only used for certain optimized cases to control the audio 
    /// source.
    pub fn chromaprint_get_num_channels(ctx: *mut ChromaprintContext) -> c_int;

    /// Get the sampling rate that is internally used for fingerprinting.
    ///
    /// You normally don't need this. Just set the audio's actual number of 
    /// channels when calling chromaprint_start() and everything will work. 
    /// This is only used for certain optimized cases to control the audio 
    /// source.
    pub fn chromaprint_get_sample_rate(ctx: *mut ChromaprintContext) -> c_int;

    /// Get the duration of one item in the raw fingerprint in samples.
    pub fn chromaprint_get_item_duration(ctx: *mut ChromaprintContext) -> c_int;


    /// Get the duration of one item in the raw fingerprint in milliseconds.
    pub fn chromaprint_get_item_duration_ms(ctx: *mut ChromaprintContext) -> c_int;

    /// Get the duration of internal buffers that the fingerprinting algorithm 
    /// uses.
    pub fn chromaprint_get_delay(ctx: *mut ChromaprintContext) -> c_int;

    /// Get the duration of internal buffers that the fingerprinting algorithm 
    /// uses.
    pub fn chromaprint_get_delay_ms(ctx: *mut ChromaprintContext) -> c_int;
    
    /// Restart the computation of a fingerprint with a new audio stream.
    ///
    /// Arguments:
    /// - `ctx`: Chromaprint context pointer
    /// - `sample_rate`: sample rate of the audio stream (in Hz)
    /// - `num_channels`: number of channels in the audio stream (1 or 2)
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_start(
        ctx: *mut ChromaprintContext,
        sample_rate: c_int,
        num_channels: c_int,
    ) -> c_int;

    /// Send audio data to the fingerprint calculator.
    ///
    /// Arguments:
    /// - `ctx`: Chromaprint context pointer
    /// - `data`: raw audio data, should point to an array of 16-bit signed 
    ///           integers in native byte-order
    /// - `size`: size of the data buffer (in samples)
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_feed(
        ctx: *mut ChromaprintContext,
        data: *const i16,
        size: c_int,
    ) -> c_int;

    /// Process any remaining buffered audio data.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_finish(ctx: *mut ChromaprintContext) -> c_int;

    /// Return the calculated fingerprint as a compressed string.
    ///
    /// The caller is responsible for freeing the returned pointer using
    /// [`chromaprint_dealloc`].
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_get_fingerprint(ctx: *mut ChromaprintContext, fingerprint: *mut *mut c_char) -> c_int;

    /// Return the calculated fingerprint as an array of 32-bit integers.
    ///
    /// The caller is responsible for freeing the returned pointer using
    /// [`chromaprint_dealloc`].
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_get_raw_fingerprint(
        ctx: *mut ChromaprintContext,
        fingerprint: *mut *mut u32,
        size: *mut c_int
    ) -> c_int;


    /// Return the length of the current raw fingerprint.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_get_raw_fingerprint_size(
        ctx: *mut ChromaprintContext,
        size: *mut c_int
    ) -> c_int;

    /// Return 32-bit hash of the calculated fingerprint.
    ///
    /// See [`chromaprint_hash_fingerprint`] for details on how to use the hash.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_get_fingerprint_hash(
        ctx: *mut ChromaprintContext,
        hash: *mut u32
    ) -> c_int;

    /// Clear the current fingerprint, but allow more data to be processed.
    ///
    /// This is useful if you are processing a long stream and want to many
    /// smaller fingerprints, instead of waiting for the entire stream to be
    /// processed.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_clear_fingerprint(ctx: *mut ChromaprintContext) -> c_int;

    /// Compress and optionally base64-encode a raw fingerprint.
    ///
    /// The caller is responsible for freeing the returned pointer using
    /// [`chromaprint_dealloc`].
    ///
    /// Arguments:
    /// - `fp`: pointer to an array of 32-bit integers representing the raw
    ///         fingerprint to be encoded
    /// - `size`: number of items in the raw fingerprint
    /// - `algorithm`: Chromaprint algorithm version which was used to generate 
    ///                the raw fingerprint
    /// - `encoded_fp`: pointer to a pointer, where the encoded fingerprint will
    ///                 be stored
    /// - `encoded_size`: size of the encoded fingerprint in bytes
    /// - `base64`: Whether to return binary data or base64-encoded ASCII data.
    ///             The compressed fingerprint will be encoded using base64 with
    ///             the URL-safe scheme if you set this parameter to 1. It will
    ///             return binary data if it's 0.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_encode_fingerprint(
        fp: *const u32,
        size: c_int,
        algorithm: ChromaprintAlgorithm,
        encoded_fp: *mut *mut c_char,
        encoded_size: *mut c_int,
        base64: c_int,
    ) -> c_int;

    /// Uncompress and optionally base64-decode an encoded fingerprint
    ///
    /// The caller is responsible for freeing the returned pointer using
    /// [`chromaprint_dealloc`].
    ///
    /// Arguments:
    /// `encoded_fp`: pointer to an encoded fingerprint
    /// `encoded_size`: size of the encoded fingerprint in bytes
    /// `fp`: pointer to a pointer, where the decoded raw fingerprint (array
    ///       of 32-bit integers) will be stored
    /// `size`: Number of items in the returned raw fingerprint
    /// `algorithm`: Chromaprint algorithm version which was used to generate 
    ///              the raw fingerprint
    /// `base64: Whether the encoded_fp parameter contains binary data or
    ///          base64-encoded ASCII data. If 1, it will base64-decode the data
    ///          before uncompressing the fingerprint.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_decode_fingerprint(
        encoded_fp: *const c_char,
        encoded_size: c_int,
        fp: *mut *mut u32,
        size: *mut c_int,
        algorithm: *mut ChromaprintAlgorithm,
        base64: c_int,
    ) -> c_int;

    /// Uncompress and optionally base64-decode an encoded fingerprint
    ///
    /// The caller is responsible for freeing the returned pointer using
    /// [`chromaprint_dealloc`].
    ///
    /// Arguments:
    /// `encoded_fp`: pointer to an encoded fingerprint
    /// `encoded_size`: size of the encoded fingerprint in bytes
    /// `size`: Number of items in the returned raw fingerprint
    /// `algorithm`: Chromaprint algorithm version which was used to generate 
    ///              the raw fingerprint
    /// `base64: Whether the encoded_fp parameter contains binary data or
    ///          base64-encoded ASCII data. If 1, it will base64-decode the data
    ///          before uncompressing the fingerprint.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_decode_fingerprint_header(
        encoded_fp: *const c_char,
        encoded_size: c_int,
        size: *mut c_int,
        algorithm: *mut ChromaprintAlgorithm,
        base64: c_int,
    ) -> c_int;

    /// Generate a single 32-bit hash for a raw fingerprint.
    /// 
    /// If two fingerprints are similar, their hashes generated by this function
    /// will also be similar. If they are significantly different, their hashes
    /// will most likely be significantly different as well, but you can't rely
    /// on that.
    /// 
    /// You compare two hashes by counting the bits in which they differ.
    /// Normally that would be something like POPCNT(hash1 XOR hash2), which 
    /// returns a number between 0 and 32. Anthing above 15 means the hashes are
    /// completely different.
    ///
    /// Returns 0 on error, 1 on success
    pub fn chromaprint_hash_fingerprint(
        fp: *const u32,
        size: c_int,
        hash: *mut u32,
    ) -> c_int;

    /// Free memory allocated by any function from the Chromaprint API.
    pub fn chromaprint_dealloc(ptr: *mut c_void);
}
