// I didn't like how the standard methods for
// allocating arrays in the heap were lacking
// (Vecs assume that they are going to change
// in size, and Boxes need to be of known
// size at compile time). 
// 
// So I wrote this simple library.

#![allow(dead_code)]

use std::{
    alloc::{self, Layout},
    mem::size_of,
    ops::{Index, IndexMut},
    fmt::{Debug, Display, Formatter}
};

#[derive(Debug)]
pub enum CajaError {
    FailedLayoutCreation,
    FailedAllocation,
    ZeroSized,
}

/// An fixed-sized array allocated in the heap
/// of any arbitrary type, and whose
/// size needn't be known at compile time.
pub struct Caja<T> {
    /// Length of the array
    length  : usize,
    data : *mut T,
}
impl<T> Caja<T> {
    pub fn new_uninitialized(size : usize) -> Result<Self, CajaError> {unsafe{
        if size == 0 {
            return Err(CajaError::ZeroSized);
        }

        // Create a layout for the allocation
        let lay = match Layout::array::<T>(size) {
            Ok(ok) => ok,
            Err(_) => return Err(CajaError::FailedLayoutCreation)
        };

        // Check that the allocation was successful
        let ptr = alloc::alloc(lay) as *mut T;
        if ptr.is_null() {
            return Err(CajaError::FailedAllocation);
        }

        return Ok(Self {
            length : size,
            data   : ptr,
        });
    };}

    pub fn new_zeroed(size : usize) -> Result<Self, CajaError> {unsafe{
        let c = match Self::new_uninitialized(size) {
            Ok(ok) => ok,
            Err(e) => return Err(e)
        };

        for i in 0..size*size_of::<T>() {
            *(c.data as *mut u8).add(i) = 0;
        }

        return Ok(c);
    };}

    /// Returns the underlying pointer in Caja
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        return self.data;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        return self.length;
    }

    pub fn as_slice(&self) -> &[T] {unsafe{
        return std::slice::from_raw_parts(self.data, self.length);
    };}

    pub fn as_mut_slice(&self) -> &mut [T] {unsafe{
        return std::slice::from_raw_parts_mut(self.data, self.length);
    };}
}
impl<T : Copy> Caja<T> {
    pub fn new(size : usize, default : T) -> Result<Self, CajaError> {unsafe{
        let c = match Self::new_uninitialized(size) {
            Ok(ok) => ok,
            Err(e) => return Err(e)
        };

        for i in 0..size {
            *c.data.add(i) = default;
        }

        return Ok(c);
    };}
}

impl<T> Drop for Caja<T> {
    fn drop(&mut self) {unsafe{
        alloc::dealloc(
            self.data as *mut u8,
            Layout::array::<T>(self.length).unwrap()
        );
    };}
}

impl<T> Index<usize> for Caja<T> {
    type Output = T;

    fn index(&self, index : usize) -> &Self::Output {unsafe{
        return self.data.add(index).as_ref().unwrap();
    };}
}
impl<T> IndexMut<usize> for Caja<T> {
    fn index_mut(&mut self, index : usize) -> &mut Self::Output {unsafe{
        return self.data.add(index).as_mut().unwrap();
    };}
}

impl<T : Copy> TryFrom<&[T]> for Caja<T> {
    type Error = CajaError;

    fn try_from(frm : &[T]) -> Result<Self,CajaError> {
        let mut ret = match Self::new_uninitialized(frm.len()) {
            Ok(ok) => ok,
            Err(e) => return Err(e)
        };
        
        for i in 0..frm.len() {
            ret[i] = frm[i];
        }

        return Ok(ret);
    }
}

impl<T : Display> Display for Caja<T> {
    fn fmt(&self, format : &mut Formatter<'_>) -> std::fmt::Result {
        let mut res = write!(format, "Length : {}\nData : [ ", self.length);
        
        for i in 0..self.length-1 {
            if res.is_err() {
                return res;
            }

            res = write!(format, "{}, ", self[i]);
        }
        res = write!(format, "{} ]\n", self[self.length-1]);

        return res;
    }
}

impl<T : Debug> Debug for Caja<T> {
    fn fmt(&self, format : &mut Formatter<'_>) -> std::fmt::Result {unsafe{
        return format.debug_struct("Caja")
            .field("length", &self.length)
            .field("data", &self.data)
            .field("data as an array", &std::slice::from_raw_parts(self.data, self.length))
            .finish();
    };}
}
