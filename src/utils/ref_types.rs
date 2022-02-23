use core::ops::Deref;

// Safety:
// pointer is not null once inited(comes from an immutable ref)
// pointee memory is always valid during the eventloop
#[repr(transparent)]
pub struct Ref<T>(*const T);

unsafe impl<T> Send for Ref<T> {}
unsafe impl<T> Sync for Ref<T> {}

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
    fn from(x: &T) -> Self {
        Ref(x as *const _)
    }
}
