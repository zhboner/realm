//! Unsafe tricks.

use core::ops::Deref;

/// A reference wrapper with static lifetime.
///
/// # Safety
///
/// Inner pointer comes from a immutable reference, which is not null.
///
/// Pointee memory must remain valid during the eventloop.
pub struct Ref<T>(*const T);

unsafe impl<T: Send + Sync> Send for Ref<T> {}
unsafe impl<T: Send + Sync> Sync for Ref<T> {}

impl<T> Copy for Ref<T> {}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> AsRef<T> for Ref<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T> From<&T> for Ref<T> {
    #[inline]
    fn from(x: &T) -> Self {
        Ref(x as *const _)
    }
}

impl<T> Ref<T> {
    #[inline]
    pub const fn new(x: &T) -> Self {
        Self(x as *const _)
    }
}
