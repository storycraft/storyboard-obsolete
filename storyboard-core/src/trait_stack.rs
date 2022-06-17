/*
 * Created on Sat Jun 04 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{
    fmt::Debug,
    marker::{PhantomData, Unsize},
    mem,
    ops::{Index, IndexMut},
    ptr::{self, DynMetadata, Pointee},
    slice,
};

#[derive(Clone)]
pub struct TraitStack<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> {
    data: Vec<u8>,
    table: Vec<(usize, DynMetadata<T>)>,
    phantom: PhantomData<T>,
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> TraitStack<T> {
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            table: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    pub fn data_capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        unsafe { Some(&*self.get_ptr(index)?) }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        unsafe { Some(&mut *(self.get_ptr(index)? as *mut T)) }
    }

    fn get_ptr(&self, index: usize) -> Option<*const T> {
        let (offset, metadata) = *self.table.get(index)?;

        Some(unsafe { self.dyn_ptr_from(offset, metadata) })
    }

    unsafe fn dyn_ptr_from(&self, offset: usize, metadata: DynMetadata<T>) -> *const T {
        let data = self.data.as_ptr().add(offset) as _;

        ptr::from_raw_parts(data, metadata)
    }

    pub fn push<I: Unsize<T>>(&mut self, item: I) {
        let (data, metadata) = (&item as *const T).to_raw_parts();

        let offset = self.data.len();

        // SAFETY: item is copied to data and original was forgotten.
        self.data
            .extend_from_slice(unsafe { slice::from_raw_parts(data as _, mem::size_of::<I>()) });
        mem::forget(item);

        self.table.push((offset, metadata));
    }

    pub fn pop(&mut self) -> Option<()> {
        let (offset, metadata) = self.table.pop()?;
        let data = unsafe { self.dyn_ptr_from(offset, metadata) };

        unsafe { ptr::drop_in_place(data as *mut T) };
        self.data.drain(offset..);

        Some(())
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.into_iter()
    }

    pub fn clear(&mut self) {
        for (offset, metadata) in &self.table {
            // SAFETY: Data and table cleared after drop
            unsafe {
                ptr::drop_in_place(self.dyn_ptr_from(*offset, *metadata) as *mut T);
            }
        }

        self.table.clear();
        self.data.clear();
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Default for TraitStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: All data stored in [TraitStack] is Send
unsafe impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + Send> Send for TraitStack<T> {}

// SAFETY: All data stored in [TraitStack] is Sync
unsafe impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + Sync> Sync for TraitStack<T> {}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + Debug> Debug for TraitStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Index<usize> for TraitStack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IndexMut<usize> for TraitStack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IntoIterator for &'a TraitStack<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            ptr: self.data.as_ptr(),
            table_iter: self.table.iter(),
        }
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IntoIterator for &'a mut TraitStack<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            ptr: self.data.as_mut_ptr(),
            table_iter: self.table.iter(),
        }
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Drop for TraitStack<T> {
    fn drop(&mut self) {
        for (offset, metadata) in &self.table {
            // SAFETY: Data and table invalid after destructor
            unsafe {
                ptr::drop_in_place(self.dyn_ptr_from(*offset, *metadata) as *mut T);
            }
        }
    }
}

pub struct Iter<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> {
    ptr: *const u8,
    table_iter: slice::Iter<'a, (usize, DynMetadata<T>)>,
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Iter<'a, T> {
    unsafe fn item_at(&self, offset: usize, metadata: DynMetadata<T>) -> &'a T {
        &*(ptr::from_raw_parts(self.ptr.add(offset) as _, metadata) as *const T)
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.table_iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.table_iter.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.nth(n)?;

        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next_back()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.nth_back(n)?;

        // SAFETY: Pointer is offseted using valid offset
        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

pub struct IterMut<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> {
    ptr: *mut u8,
    table_iter: slice::Iter<'a, (usize, DynMetadata<T>)>,
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IterMut<'a, T> {
    unsafe fn item_at(&mut self, offset: usize, metadata: DynMetadata<T>) -> &'a mut T {
        &mut *(ptr::from_raw_parts::<T>(self.ptr.add(offset) as _, metadata) as *mut T)
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.table_iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.table_iter.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.nth(n)?;

        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next_back()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.nth_back(n)?;

        // SAFETY: Pointer is offseted using valid offset
        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

#[cfg(test)]
mod tests {
    use super::TraitStack;
    use std::fmt::Debug;

    #[test]
    fn trait_stack_test() {
        let mut stack = TraitStack::<dyn Debug>::new();

        stack.push("str");
        stack.push(1);
        stack.push(28342.2);
        stack.push("String".to_string());

        println!("{:?}", stack);
    }
}
