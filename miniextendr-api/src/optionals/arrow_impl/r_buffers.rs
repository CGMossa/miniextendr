//! Ownership records for R-backed Arrow buffers.
//!
//! Buffer capacity and bytes before its data pointer cannot identify an R
//! allocation. Only buffers registered while borrowing a real, non-ALTREP R
//! vector may recover that vector. Each independent Arrow allocation owner
//! keeps one R preserve root until its last clone is dropped.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use arrow_buffer::Buffer;

use crate::worker::Sendable;
use crate::{RNativeType, SEXP, SEXPTYPE};

struct Entry {
    source: Sendable<SEXP>,
    sexptype: SEXPTYPE,
    len: usize,
    owners: usize,
}

#[derive(Default)]
struct RBuffers {
    entries: HashMap<usize, Entry>,
    pending_release: Vec<Sendable<SEXP>>,
}

impl RBuffers {
    fn register(&mut self, key: usize, source: SEXP, sexptype: SEXPTYPE, len: usize) {
        self.entries
            .entry(key)
            .and_modify(|entry| entry.owners += 1)
            .or_insert(Entry {
                source: Sendable(source),
                sexptype,
                len,
                owners: 1,
            });
    }

    fn unregister(&mut self, key: usize) {
        let entry = self.entries.get_mut(&key).unwrap();
        entry.owners -= 1;
        if entry.owners == 0 {
            self.entries.remove(&key);
        }
    }

    fn lookup<T: RNativeType>(&self, buffer: &Buffer, len: usize) -> Option<SEXP> {
        if len == 0 || buffer.ptr_offset() != 0 {
            return None;
        }
        let entry = self.entries.get(&buffer.as_ptr().addr())?;
        (entry.sexptype == T::SEXP_TYPE && entry.len == len).then_some(entry.source.0)
    }
}

fn buffers() -> &'static Mutex<RBuffers> {
    static BUFFERS: OnceLock<Mutex<RBuffers>> = OnceLock::new();
    BUFFERS.get_or_init(|| Mutex::new(RBuffers::default()))
}

/// The allocation owner may be dropped on any thread. R access is confined to
/// construction and main-thread destruction; other drops queue their root.
pub(super) struct RBufferOwner {
    source: Sendable<SEXP>,
    key: Option<usize>,
}

// SAFETY: the SEXP is only used by R's main thread. Background threads only
// enqueue the pointer for release and update Rust-owned metadata under a mutex.
unsafe impl Sync for RBufferOwner {}
impl std::panic::RefUnwindSafe for RBufferOwner {}

impl RBufferOwner {
    /// `source` must stay caller-rooted across this main-thread construction.
    pub(super) unsafe fn new<T: RNativeType>(source: SEXP, ptr: *const u8) -> Self {
        use crate::SexpExt;

        unsafe { crate::sys::R_PreserveObject(source) };
        // An ALTREP can expose another object's storage. Keeping its preserve
        // root is necessary, but it does not establish unique buffer identity.
        let key = (!source.is_altrep()).then_some(ptr.addr());
        if let Some(key) = key {
            let len = source.len();
            buffers()
                .lock()
                .unwrap()
                .register(key, source, T::SEXP_TYPE, len);
        }
        Self {
            source: Sendable(source),
            key,
        }
    }
}

impl Drop for RBufferOwner {
    fn drop(&mut self) {
        let on_main = crate::worker::is_r_main_thread();
        {
            let mut state = buffers().lock().unwrap();
            if let Some(key) = self.key {
                state.unregister(key);
            }
            if !on_main {
                state.pending_release.push(Sendable(self.source.0));
            }
        }
        if on_main {
            unsafe { crate::sys::R_ReleaseObject(self.source.0) };
        }
    }
}

pub(super) fn original_r_vector<T: RNativeType>(buffer: &Buffer, len: usize) -> Option<SEXP> {
    buffers().lock().unwrap().lookup::<T>(buffer, len)
}

/// Release roots queued by background threads at an R unwind boundary.
/// Never hold the Rust mutex while calling R: R finalizers can drop buffers.
pub(crate) fn drain_pending_r_buffer_releases() {
    let pending = std::mem::take(&mut buffers().lock().unwrap().pending_release);
    for source in pending {
        unsafe { crate::sys::R_ReleaseObject(source.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregistered_exact_capacity_buffer_is_not_recovered() {
        let buffer = Buffer::from(vec![0u8; 8]);
        assert_eq!(buffer.ptr_offset(), 0);
        assert_eq!(buffer.capacity(), 8);
        assert!(RBuffers::default().lookup::<f64>(&buffer, 1).is_none());
    }

    #[test]
    fn registrations_follow_independent_owners_and_reject_changed_views() {
        let buffer = Buffer::from(vec![0u8; 16]);
        let key = buffer.as_ptr().addr();
        // A sentinel handle is enough: ownership lookup must never read it.
        let source = SEXP(std::ptr::null_mut());
        let mut state = RBuffers::default();
        state.register(key, source, SEXPTYPE::REALSXP, 2);
        state.register(key, source, SEXPTYPE::REALSXP, 2);
        assert_eq!(state.lookup::<f64>(&buffer.clone(), 2), Some(source));
        assert!(state.lookup::<i32>(&buffer, 2).is_none());
        assert!(
            state
                .lookup::<f64>(&buffer.slice_with_length(0, 8), 1)
                .is_none()
        );
        assert!(state.lookup::<f64>(&buffer.slice(8), 1).is_none());
        state.unregister(key);
        assert_eq!(state.lookup::<f64>(&buffer, 2), Some(source));
        state.unregister(key);
        assert!(state.lookup::<f64>(&buffer, 2).is_none());
    }
}
