use crate::{
    std::{
        borrow,
        marker::PhantomData,
    },
    stream::stack::Stack,
};

use super::Error;

/**
Similar to `BorrowMut<T>`.

This trait produces a `DebugRefMut<'a T>` instead of a `&'a mut T`,
which is zero-sized in release mode.
*/
pub(crate) trait DebugBorrowMut<T: ?Sized> {
    fn borrow_mut(&mut self) -> DebugRefMut<T>;
}

impl<T: ?Sized> DebugBorrowMut<T> for T
where
    T: borrow::BorrowMut<T>,
{
    fn borrow_mut(&mut self) -> DebugRefMut<T> {
        DebugRefMut {
            #[cfg(debug_assertions)]
            ref_mut: borrow::BorrowMut::borrow_mut(self),
            _m: Default::default(),
        }
    }
}

pub(crate) struct DebugRefMut<'a, T: ?Sized> {
    #[cfg(debug_assertions)]
    ref_mut: &'a mut T,
    _m: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> DebugBorrowMut<T> for DebugRefMut<'a, T> {
    fn borrow_mut(&mut self) -> DebugRefMut<T> {
        DebugRefMut {
            #[cfg(debug_assertions)]
            ref_mut: self.ref_mut,
            _m: Default::default(),
        }
    }
}

/**
An internal stack that provides additional validation in debug
builds for callers of `Stream`. It ensures:

- Only valid calls to the stream are made.
- The stream is not re-used if it returns an error.

These checks aren't exactly bullet-proof, but can assist consumers
in holding streams correctly during development.
*/
#[derive(Default)]
pub(crate) struct DebugStack {
    #[cfg(debug_assertions)]
    inner: DebugStackInner,
    #[cfg(debug_assertions)]
    poisoned: bool,
}

#[derive(Default)]
pub(crate) struct DebugStackInner {
    #[cfg(debug_assertions)]
    stack: Stack,
    _m: PhantomData<Stack>,
}

impl<'a> DebugRefMut<'a, DebugStack> {
    #[inline]
    pub(crate) fn and_then(
        &mut self,
        mut f: impl FnMut(DebugRefMut<DebugStackInner>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                if self.ref_mut.poisoned {
                    return Err(Error::msg("attempt to use a poisoned stream"));
                }

                self.ref_mut.poisoned = true;
                f(self.ref_mut.inner.borrow_mut())?;
                self.ref_mut.poisoned = false;

                Ok(())
            } else {
                f(DebugStackInner::default().borrow_mut())
            }
        }
    }
}

impl<'a> DebugRefMut<'a, DebugStackInner> {
    #[inline]
    pub fn begin(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.begin()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn primitive(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.primitive()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn map_begin(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.map_begin()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn map_key(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.map_key()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn map_value(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.map_value()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn map_end(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.map_end()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn seq_begin(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.seq_begin()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn seq_elem(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.seq_elem()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn seq_end(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.seq_end()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
        cfg_debug_stack! {
            if #[debug_assertions] {
                self.ref_mut.stack.end()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(debug_assertions))]
    mod release {
        use super::super::*;

        use crate::std::mem;

        #[test]
        fn debug_stack_is_zero_sized() {
            assert_eq!(0, mem::size_of::<DebugStack>());
        }

        #[test]
        fn debug_stack_ref_is_zero_sized() {
            assert_eq!(0, mem::size_of::<DebugRefMut<DebugStack>>());
        }
    }
}
