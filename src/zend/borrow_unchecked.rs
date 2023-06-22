use crate::binary_slice::{BinarySlice, PackSlice};

#[inline(always)]
pub unsafe fn borrow_unchecked<
    'original,
    'unbounded,
    Ref: BorrowUnchecked<'original, 'unbounded>,
>(
    reference: Ref,
) -> Ref::Unbounded {
    unsafe { BorrowUnchecked::borrow_unchecked(reference) }
}

#[doc(hidden)]
pub unsafe trait BorrowUnchecked<'original, 'unbounded> {
    type Unbounded;

    unsafe fn borrow_unchecked(self) -> Self::Unbounded;
}

unsafe impl<'original, 'unbounded, T: 'unbounded> BorrowUnchecked<'original, 'unbounded>
    for &'original T
{
    type Unbounded = &'unbounded T;

    #[inline(always)]
    unsafe fn borrow_unchecked(self) -> Self::Unbounded {
        unsafe { ::core::mem::transmute(self) }
    }
}

unsafe impl<'original, 'unbounded, T: 'unbounded> BorrowUnchecked<'original, 'unbounded>
    for &'original mut T
{
    type Unbounded = &'unbounded mut T;

    #[inline(always)]
    unsafe fn borrow_unchecked(self) -> Self::Unbounded {
        unsafe { ::core::mem::transmute(self) }
    }
}

unsafe impl<'original, 'unbounded, T: 'unbounded + PackSlice> BorrowUnchecked<'original, 'unbounded>
    for BinarySlice<'original, T>
{
    type Unbounded = BinarySlice<'unbounded, T>;

    #[inline(always)]
    unsafe fn borrow_unchecked(self) -> Self::Unbounded {
        unsafe { ::core::mem::transmute(self) }
    }
}