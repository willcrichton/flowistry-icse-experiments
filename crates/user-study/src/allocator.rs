use std::cell::RefCell;
use std::mem;

/// Bump allocator that starts with a fixed capacity and
/// allocates pointers until the capacity is exhausted.
pub struct Allocator(RefCell<AllocatorInner>);
struct AllocatorInner {
  memory: Vec<u8>,
  index: usize,
}

impl AllocatorInner {
  /// Get the next index in `memory` that satisfies alignment for `T`.
  fn next_aligned<T>(&self) -> usize {
    let align = mem::align_of::<T>();
    (self.index + align - 1) / align * align
  }

  /// Manufactures a pointer at `index` and bumps to the next free space.
  unsafe fn get_and_bump<'a, 'b, T>(&'a mut self, index: usize) -> &'b mut T {
    let size = mem::size_of::<T>();
    let ptr = self.memory.as_mut_ptr().add(index);
    self.index = index + size;
    &mut *(ptr as *mut T)
  }
}

impl Allocator {
  pub fn new(capacity: usize) -> Self {
    Allocator(RefCell::new(AllocatorInner {
      memory: Vec::with_capacity(capacity),
      index: 0,
    }))
  }

  /// Allocates a pointer for T, returning None if not enough space is left.
  pub fn allocate<T>(&self, aligned: bool) -> Option<&mut T> {
    let mut inner = self.0.borrow_mut();
    let next = if aligned {
      inner.next_aligned::<T>()
    } else {
      inner.index
    };
    if next >= inner.memory.capacity() {
      return None;
    }
    Some(unsafe { inner.get_and_bump(next) })
  }
}

#[test]
fn alloc_test1() {
  let size = mem::size_of::<usize>();
  let alloc = Allocator::new(size * 10);
  let ns = (0..5)
    .map(|i| {
      let n = alloc.allocate::<usize>(true).unwrap();
      *n = i;
      n
    })
    .collect::<Vec<_>>();
  assert_eq!(ns, [&mut 0, &mut 1, &mut 2, &mut 3, &mut 4]);
}

#[test]
fn alloc_test2() {
  let size = mem::size_of::<usize>();
  let alloc = Allocator::new(size + 1);
  let _n1 = alloc.allocate::<usize>(true).unwrap();
  let n2 = alloc.allocate::<usize>(true);
  assert!(n2.is_none());
}
