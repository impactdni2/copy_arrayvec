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
    pub fn len(&self) -> usize {
        self.len
    }
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Push a new element
    ///
    /// # Panics
    /// If the [`CopyArrayVec`] is full
    ///
    /// # Complexity
    /// O(1)
    pub fn push(&mut self, el: T) {
        assert!(self.len() < MAX, "tried to push to full arrayvec");

        let next = self.len;
        self.buf[next].write(el);
        self.len += 1;
    }
    /// Pop an element from the back
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(self.len - 1))
        }
    }
    /// Remove an element from a specific position
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
    /// Has the same complexity bounds as [`CopyArrayVec::remove`]
    pub fn insert(&mut self, i: usize, value: T) {
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
    pub fn len_remaining(&self) -> usize {
        MAX - self.len()
    }

    /// Remove all elements
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
        assert_eq!(arr.deref(), (0..20).map(|x| x * x).collect::<Vec<usize>>().deref());
    }

    #[test]
    fn remove_at_start() {
        let mut arr = upto_vec::<10>();
        arr.remove(0);
        assert_eq!(arr, upto_vec::<10>().iter().skip(1).copied().collect::<CopyArrayVec<_, 10>>());
    }
}