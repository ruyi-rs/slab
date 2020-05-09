//! Provides an object based allocator [`Slab<T>`] backed by a contiguous
//! growable array of slots.
//!
//! The slab allocator pre-allocates memory for objects of same type so that
//! it reduces fragmentation caused by allocations and deallocations. When
//! allocating memory for an object, it just finds a free (unused) slot, marks
//! it as used, and returns the index of the slot for later access to the
//! object. When freeing an object, it just adds the slot holding the object
//! to the list of free (unused) slots after dropping the object.
//!
//! # Examples
//!
//! ```
//! # use ruyi_slab::Slab;
//! // Explicitly create a Slab<T> with new
//! let mut slab = Slab::new();
//!
//! // Insert an object into the slab
//! let one = slab.insert(1);
//!
//! // Remove an object at the specified index
//! let removed = slab.remove(one);
//!
//! assert_eq!(removed.unwrap(), 1);
//! ```
//!
//! [`Slab<T>`]: struct.Slab.html

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::vec::Vec;

use core::fmt;
use core::mem;
use core::ops::{Index, IndexMut};

#[cfg(debug_assertions)]
#[inline]
fn unreachable() -> ! {
    unreachable!()
}

#[cfg(not(debug_assertions))]
unsafe fn unreachable() -> ! {
    core::hint::unreachable_unchecked()
}

enum Slot<T> {
    Used(T),
    Free(usize),
}

impl<T> Slot<T> {
    #[inline]
    unsafe fn get_unchecked(&self) -> &T {
        match self {
            Slot::Used(obj) => obj,
            Slot::Free(_) => unreachable(),
        }
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        match self {
            Slot::Used(obj) => obj,
            Slot::Free(_) => unreachable(),
        }
    }

    #[inline]
    unsafe fn get_free_unchecked(&self) -> usize {
        match self {
            Slot::Free(index) => *index,
            Slot::Used(_) => unreachable(),
        }
    }

    #[inline]
    unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Slot::Used(obj) => obj,
            Slot::Free(_) => unreachable(),
        }
    }

    #[inline]
    unsafe fn take(&mut self, index: usize) -> T {
        mem::replace(self, Slot::Free(index)).unwrap_unchecked()
    }

    #[inline]
    unsafe fn put(&mut self, obj: T) -> usize {
        mem::replace(self, Slot::Used(obj)).get_free_unchecked()
    }
}

impl<T: fmt::Debug> fmt::Debug for Slot<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Used(obj) => write!(f, "Used({:?}", obj),
            Slot::Free(index) => write!(f, "Free({})", index),
        }
    }
}

/// An object based allocator backed by a contiguous growable array of slots.
///
/// # Examples
/// ```
/// # use ruyi_slab::Slab;
/// let mut slab = Slab::new();
/// let one = slab.insert(1);
/// let two = slab.insert(2);
///
/// assert_eq!(slab.len(), 2);
/// assert_eq!(slab[two], 2);
///
/// slab.remove(one);
///
/// assert_eq!(slab.len(), 1);
///
/// let entry = slab.free_entry();
/// let index = entry.index();
/// entry.insert(index);
///
/// assert_eq!(slab.len(), 2);
/// assert_eq!(slab[index], index);
/// ```
///
/// # Capacity and reallocation
///
/// The capacity of a slab is the amount of space allocated for any future
/// objects that will be inserted to the slab. This is not to be confused with
/// the *length* of a slab, which specifies the number of actual objects
/// within the slab. If a slab's length exceeds its capacity, its capacity
/// will automatically be increased, but its objects will have to be
/// reallocated.
///
/// For example, a slab with capacity 10 and length 0 would be an empty slab
/// with space for 10 more objects. Inserting 10 or fewer objects into the
/// slab will not change its capacity or cause reallocation to occur. However,
/// if the slab's length is increased to 11, it will have to reallocate, which
/// can be slow. For this reason, it is recommended to use [`Slab::with_capacity`]
/// whenever possible to specify how big the slab is expected to get.
///
/// [`Slab::with_capacity`]: #method.with_capacity
#[derive(Debug)]
pub struct Slab<T> {
    slots: Vec<Slot<T>>,
    len: usize,
    free: usize,
}

unsafe impl<T: Send> Send for Slab<T> {}

impl<T> Slab<T> {
    const NULL: usize = core::usize::MAX;

    /// Constructs a new empty `Slab<T>`.
    /// The allocator will not allocate until the first object is inserted.
    ///
    /// # Examples
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::new();
    /// # slab.insert(1);
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            len: 0,
            free: Self::NULL,
        }
    }

    /// Constructs a new, empty `Slab<T>` with the specified capacity.
    ///
    /// The slab will be able to hold exactly `capacity` objects without
    /// reallocating. If `capacity` is 0, the slab will not allocate.
    ///
    /// It is important to note that although the returned slab has the
    /// *capacity* specified, the slab will have a zero *length*. For an
    /// explanation of the difference between length and capacity, see
    /// *[Capacity and reallocation]*.
    ///
    /// [Capacity and reallocation]: #capacity-and-reallocation
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(10);
    ///
    /// // The slab contains no objects, even though it has capacity for more
    /// assert_eq!(slab.len(), 0);
    ///
    /// // These are all done without reallocating...
    /// for i in 0..10 {
    ///     slab.insert(i);
    /// }
    ///
    /// // ...but this may make the slab reallocate
    /// slab.insert(11);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            len: 0,
            free: Self::NULL,
        }
    }

    /// Returns the number of objects in the slab.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(3);
    /// slab.insert(1);
    /// slab.insert(2);
    /// slab.insert(3);
    ///
    /// assert_eq!(slab.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the number of objects the slab can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let slab: Slab<i32> = Slab::with_capacity(10);
    ///
    /// assert_eq!(slab.capacity(), 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Returns `true` if the slab contains no objects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::new();
    ///
    /// assert!(slab.is_empty());
    ///
    /// slab.insert(1);
    ///
    /// assert!(!slab.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the slab, removing all objects.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the slab.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(3);
    /// slab.insert(1);
    /// slab.insert(2);
    /// slab.clear();
    ///
    /// assert!(slab.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        if self.len > 0 {
            self.slots.clear();
            self.len = 0;
            self.free = Self::NULL;
        } else {
            unsafe {
                self.slots.set_len(0);
            }
        }
    }

    /// Reserves capacity for at least `additional` more objects to be inserted
    /// in the given `Slab<T>`. The slab may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(1);
    /// slab.insert(1);
    /// slab.reserve(10);
    ///
    /// assert!(slab.capacity() >= 11);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let n = self.slots.capacity() - self.len;
        if additional > n {
            self.slots.reserve(additional - n);
        }
    }

    /// Reserves the minimum capacity for exactly `additional` more objects to
    /// be inserted in the given `Slab<T>`. After calling `reserve_exact`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer `reserve` if future insertions are expected.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows`usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(1);
    /// slab.insert(1);
    /// slab.reserve_exact(10);
    ///
    /// assert!(slab.capacity() >= 11);
    /// ```
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        let n = self.slots.capacity() - self.len;
        if additional > n {
            self.slots.reserve_exact(additional - n);
        }
    }

    /// Inserts an object to the slab.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(3);
    /// slab.insert(1);
    /// slab.insert(2);
    ///
    /// assert_eq!(slab.len(), 2);
    ///
    /// slab.insert(3);
    ///
    /// assert_eq!(slab.len(), 3);
    /// ```
    #[inline]
    pub fn insert(&mut self, obj: T) -> usize {
        let cur;
        if self.has_free_slots() {
            cur = self.free;
            self.free = unsafe { self.slots.get_unchecked_mut(cur).put(obj) };
        } else {
            cur = self.len;
            self.slots.push(Slot::Used(obj));
        }
        self.len += 1;
        cur
    }

    /// Returns an entry referring to an unused slot for further manipulation.
    /// It is useful when an object to be inserted need know its slab index.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(1);
    /// let entry = slab.free_entry();
    /// let index = entry.index();
    /// let obj = (index, "My slab index");
    /// entry.insert(obj);
    ///
    /// assert_eq!(slab[index].0, index);
    #[inline]
    pub fn free_entry(&mut self) -> Entry<'_, T> {
        Entry::new(self)
    }

    /// Removes and returns the object at the specified `index`, and the slot
    /// will be put to the list of free slots for reusing. `None` is returned
    /// if no object is found at the specified `index`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(1);
    /// let one = slab.insert(1);
    ///
    /// assert_eq!(slab.len(), 1);
    /// assert_eq!(slab.remove(one).unwrap(), 1);
    /// assert!(slab.is_empty());
    /// ```
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if let Some(slot) = self.slots.get_mut(index) {
            if let Slot::Used(_) = slot {
                let obj = unsafe { slot.take(self.free) };
                self.free = index;
                self.len -= 1;
                return Some(obj);
            }
        }
        None
    }

    /// Returns a reference to the object at the specified `index` if the
    /// object exists. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(3);
    /// let one = slab.insert(1);
    /// let two = slab.insert(2);
    /// let three = slab.insert(3);
    ///
    /// assert_eq!(slab.get(one), Some(&1));
    /// assert_eq!(slab.get(three), Some(&3));
    /// assert_eq!(slab.get(slab.len()), None);
    ///
    /// slab.remove(two);
    ///
    /// assert_eq!(slab.get(two), None);
    /// assert_eq!(slab.get(slab.capacity()), None);
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if let Some(slot) = self.slots.get(index) {
            if let Slot::Used(obj) = slot {
                return Some(obj);
            }
        }
        None
    }

    /// Returns a mutable reference to the object at the specified `index`
    /// if the object exists. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(2);
    /// let one = slab.insert(1);
    /// let two = slab.insert(2);
    ///
    /// assert_eq!(slab[one], 1);
    /// assert_eq!(slab[two], 2);
    ///
    /// *slab.get_mut(one).unwrap() = 3;
    /// slab.remove(two);
    ///
    /// assert_eq!(slab[one], 3);
    /// assert_eq!(slab.get_mut(two), None);
    /// ```
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(slot) = self.slots.get_mut(index) {
            if let Slot::Used(obj) = slot {
                return Some(obj);
            }
        }
        None
    }

    /// Removes and returns the object at the specified `index` without
    /// checking if the object exists or not.
    ///
    /// # Safety
    ///
    /// If the slot at the specified `index` does not have an object, the
    /// behavior of calling this method is undefined.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(1);
    /// let one = slab.insert(1);
    ///
    /// assert_eq!(slab.len(), 1);
    /// unsafe {
    ///     assert_eq!(slab.remove_unchecked(one), 1);
    /// }
    /// assert!(slab.is_empty());
    /// ```
    #[inline]
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        let obj = self.slots.get_unchecked_mut(index).take(self.free);
        self.free = index;
        self.len -= 1;
        obj
    }

    /// Returns a reference to the object at the specified `index` without
    /// checking if the object exists or not.
    ///
    /// # Safety
    ///
    /// If the slot at the specified `index` does not have an object, the
    /// behavior of calling this method is undefined even if the resulting
    /// reference is not used.
    ///
    /// For a safe alternative see [`get`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(1);
    /// let one = slab.insert(1);
    ///
    /// unsafe {
    ///     assert_eq!(slab.get_unchecked(one), &1);
    /// }
    /// ```
    ///
    /// [`get`]: #method.get
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.slots.get_unchecked(index).get_unchecked()
    }

    /// Returns a mutable reference to the object at the specified `index`
    /// without checking if the object exists or not.
    ///
    /// # Safety
    ///
    /// If the slot at the specified `index` does not have an object, the
    /// behavior of calling this method is undefined even if the resulting
    /// reference is not used.
    ///
    /// For a safe alternative see [`get_mut`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(1);
    /// let one = slab.insert(1);
    ///
    /// assert_eq!(slab[one], 1);
    ///
    /// unsafe {
    ///     *slab.get_unchecked_mut(one) = 2;
    /// }
    ///
    /// assert_eq!(slab[one], 2);
    /// ```
    ///
    /// [`get_mut`]: #method.get_mut
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.slots.get_unchecked_mut(index).get_unchecked_mut()
    }

    #[inline]
    fn has_free_slots(&self) -> bool {
        self.free != Self::NULL
    }

    #[inline]
    fn next_free(&self) -> usize {
        if self.has_free_slots() {
            self.free
        } else {
            self.len
        }
    }
}

impl<T> Default for Slab<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Slab<T> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> Index<usize> for Slab<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Some(obj) => obj,
            None => panic!("invalid slab index {}", index),
        }
    }
}

impl<T> IndexMut<usize> for Slab<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(obj) => obj,
            None => panic!("invalid slab index {}", index),
        }
    }
}

/// A handle to a free slot in a `Slab<T>`.
#[derive(Debug)]
pub struct Entry<'a, T> {
    slab: &'a mut Slab<T>,
}

impl<'a, T> Entry<'a, T> {
    #[inline]
    fn new(slab: &'a mut Slab<T>) -> Self {
        Self { slab }
    }

    /// Returns the index of the free slot that this entry refers to.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(2);
    /// slab.insert(1);
    /// let entry = slab.free_entry();
    /// let index = entry.index();
    /// entry.insert(index);
    ///
    /// assert_eq!(slab.len(), 2);
    /// assert_eq!(slab[index], index);
    /// ```
    #[inline]
    pub fn index(&self) -> usize {
        self.slab.next_free()
    }

    /// Inserts the specified object into the slot this entry refers to.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruyi_slab::Slab;
    /// let mut slab = Slab::with_capacity(2);
    /// slab.insert(1);
    /// let entry = slab.free_entry();
    /// let index = entry.index();
    /// entry.insert(index);
    ///
    /// assert_eq!(slab.len(), 2);
    /// assert_eq!(slab[index], index);
    /// ```
    #[inline]
    pub fn insert(self, obj: T) {
        self.slab.insert(obj);
    }
}
