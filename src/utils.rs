//! Utilities functions to ease life of end users.
use std;

/// Fuse contiguous slices together back into one.
/// This panics if slices are not contiguous.
pub fn fuse_slices<'a, 'b, 'c: 'a + 'b, T: 'c>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.add(s1.len()) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

/// iterate on starting_value * 2**i
pub fn powers(starting_value: usize) -> impl Iterator<Item = usize> {
    (0..).scan(starting_value, |state, _| {
        *state *= 2;
        Some(*state)
    })
}
