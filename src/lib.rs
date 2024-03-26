use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Copy)]
pub struct CopyArrayVec<T: Copy, const MAX: usize> {
    buf: [MaybeUninit<T>; MAX],
    len: usize,
}
impl<T: Copy + std::fmt::Debug, const MAX: usize> std::fmt::Debug for CopyArrayVec<T, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CopyArrayVec")
            .field("max", &MAX)
            .field("buf", &self.deref())
            .finish()
    }
}

impl<T: Copy, const MAX: usize> Default for CopyArrayVec<T, MAX> {
    fn default() -> Self {
        Self {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
}

impl<T: Copy, const MAX: usize> CopyArrayVec<T, MAX> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Get the length
    pub const fn len(&self) -> usize {
        self.len
    }
    /// Check if empty
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Push a new element
    ///
    /// # Panics
    /// If the [`CopyArrayVec`] is full
    ///
    /// ```should_panic
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 0>::new();
    /// arr.push(5);
    /// ```
    ///
    /// # Complexity
    /// O(1)
    ///
    ///
    pub fn push(&mut self, el: T) {
        assert!(self.len() < MAX, "tried to push to full arrayvec");

        let next = self.len;
        self.buf[next].write(el);
        self.len += 1;
    }

    /// Attempt to push a new element
    ///
    /// This will return an Err if the [`CopyArrayVec`] is full
    ///
    /// ```rust
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 1>::new();
    /// arr.push(5);
    /// assert_eq!(arr.try_push(0), Err(0));
    /// ```

    pub fn try_push(&mut self, el: T) -> Result<(), T> {
        if self.capacity_remaining() > 0 {
            self.push(el);
            Ok(())
        } else {
            Err(el)
        }
    }

    /// Pop an element from the back
    ///
    /// ```rust
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 1>::new();
    /// arr.push(1);
    /// assert_eq!(arr.pop(), Some(1));
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(self.len - 1))
        }
    }
    /// Remove an element from a specific position
    ///
    /// ```rust
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 5>::new();
    /// arr.push(4);
    /// arr.push(2);
    /// arr.push(5);
    ///
    /// assert_eq!(arr.remove(1), 2);
    /// assert_eq!(arr[0], 4);
    /// assert_eq!(arr[1], 5);
    /// ```
    ///
    /// # Panics
    /// If `i` is out of range
    ///
    /// ```should_panic
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 5>::new();
    /// arr.push(4);
    /// arr.remove(1);
    /// ```
    ///
    ///
    /// # Complexity Notes
    /// This is _technically_ O(n) worst case algorithmically
    /// but it is a single memcpy in reality due to the [`Copy`] bound
    pub fn remove(&mut self, i: usize) -> T {
        let v = self[i];
        unsafe {
            let buf_p = self.buf.as_mut_ptr().add(i);
            std::ptr::copy(buf_p.add(1).cast_const(), buf_p, self.len - i)
        }
        self.len -= 1;
        v
    }
    /// Insert an element at a specific position
    ///
    /// # Panics
    /// If `i` is out of bounds or if the [`CopyArrayVec`] is full
    ///
    /// ```should_panic
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 1>::new();
    /// arr.insert(3, 0);
    /// ```
    ///
    /// ```should_panic
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 1>::new();
    /// arr.push(4);
    /// arr.insert(0, 2);
    /// ```
    ///
    /// # Complexity
    /// Has the same complexity bounds as [`CopyArrayVec::remove`]
    pub fn insert(&mut self, i: usize, value: T) {
        assert!(!self.is_full(), "tried to insert into a full CopyArrayVec");
        if i == self.len() {
            self.push(value);
        } else {
            unsafe {
                let buf_p = self.buf.as_mut_ptr().add(i);
                std::ptr::copy(buf_p.cast_const(), buf_p.add(1), self.len - i);
            }
            self.len += 1;
        }
    }

    /// Try to insert and error on full
    ///
    /// ```
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 1>::new();
    /// arr.push(3);
    /// assert_eq!(arr.try_insert(0, 4), Err(4));
    /// ```
    ///
    /// # Panics
    /// If `i` is out of bounds

    pub fn try_insert(&mut self, i: usize, value: T) -> Result<(), T> {
        if self.is_full() {
            Err(value)
        } else {
            self.insert(i, value);
            Ok(())
        }
    }
    /// The remaining capacity of the [`CopyArrayVec`]
    ///
    /// ```
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 5>::new();
    /// assert_eq!(arr.capacity_remaining(), 5);
    /// arr.push(2);
    /// assert_eq!(arr.capacity_remaining(), 4);
    /// ```
    pub const fn capacity_remaining(&self) -> usize {
        MAX - self.len()
    }

    /// Check if the [`CopyArrayVec`] is full
    ///
    /// ```
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 2>::new();
    /// arr.push(0);
    /// assert!(!arr.is_full());
    /// arr.push(1);
    /// assert!(arr.is_full());
    /// ```
    pub const fn is_full(&self) -> bool {
        self.capacity_remaining() == 0
    }
    /// The max capacity of the [`CopyArrayVec`]
    ///
    /// ```
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<usize, 2>::new();
    /// assert_eq!(arr.capacity(), 2);
    /// ```
    pub const fn capacity(&self) -> usize {
        MAX
    }

    /// Remove all elements
    ///
    /// ```
    /// # use copy_arrayvec::CopyArrayVec;
    /// let mut arr = CopyArrayVec::<_, 3>::new();
    /// arr.push(2);
    /// arr.push(3);
    /// assert_eq!(arr.len(), 2);
    /// arr.clear();
    /// assert_eq!(arr.len(), 0);
    /// ```
    ///
    /// # Complexity
    /// This is an O(1) operation as it does not have
    /// to drop anything
    pub fn clear(&mut self) {
        // this is trivial because we know that `T` does not require drop we can just
        // reset our write head
        self.len = 0;
    }
}

impl<T: Copy, const MAX: usize> Deref for CopyArrayVec<T, MAX> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.buf.as_ptr().cast(), self.len()) }
    }
}

impl<T: Copy, const MAX: usize> DerefMut for CopyArrayVec<T, MAX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.buf.as_mut_ptr().cast(), self.len()) }
    }
}
impl<T: Copy, const MAX: usize> Extend<T> for CopyArrayVec<T, MAX> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T: Copy + PartialEq, const MAX: usize> PartialEq for CopyArrayVec<T, MAX> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<T: Copy + Eq, const MAX: usize> Eq for CopyArrayVec<T, MAX> {}

impl<T: Copy + std::hash::Hash, const MAX: usize> std::hash::Hash for CopyArrayVec<T, MAX> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T: Copy, const MAX: usize> FromIterator<T> for CopyArrayVec<T, MAX> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut me = Self::default();
        for item in iter {
            me.push(item);
        }
        me
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::CopyArrayVec;

    fn upto_vec<const M: usize>() -> CopyArrayVec<usize, M> {
        (0..M).collect()
    }

    #[test]
    fn create_and_push() {
        let mut arr = CopyArrayVec::<_, 10>::new();
        arr.push(5);
        arr.push(3);
        arr.push(1);
    }

    #[test]
    fn create_and_pop() {
        let mut arr = CopyArrayVec::<_, 4>::new();
        arr.push(5);
        arr.push(1);
        assert_eq!(arr.pop(), Some(1));
        assert_eq!(arr.pop(), Some(5));
        assert_eq!(arr.len(), 0);
    }

    #[test]
    #[should_panic(expected = "tried to push to full arrayvec")]
    fn pushing_to_full_panics() {
        let mut arr = CopyArrayVec::<_, 1>::new();
        arr.push(0);
        arr.push(1);
    }

    #[test]
    fn iterate() {
        let arr = (0..20).collect::<CopyArrayVec<usize, 20>>();
        for (i, el) in arr.iter().enumerate() {
            assert_eq!(i, *el);
        }
    }

    #[test]
    fn iterate_mut() {
        let mut arr = (0..20).collect::<CopyArrayVec<usize, 20>>();
        for (i, el) in arr.iter_mut().enumerate() {
            *el *= i;
        }
        assert_eq!(
            arr.deref(),
            (0..20).map(|x| x * x).collect::<Vec<usize>>().deref()
        );
    }

    #[test]
    fn remove_at_start() {
        let mut arr = upto_vec::<10>();
        arr.remove(0);
        assert_eq!(
            arr,
            upto_vec::<10>()
                .iter()
                .skip(1)
                .copied()
                .collect::<CopyArrayVec<_, 10>>()
        );
    }
}
