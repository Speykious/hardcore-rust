use core::cell::Cell;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr::{slice_from_raw_parts_mut, NonNull};

use crate::os::{self, VirtualMemory};

pub struct Arena {
	curr_offset: Cell<usize>,
	uncommitted_offset: Cell<usize>,
	vm: VirtualMemory,
}

impl Arena {
	/// Instantiates a new arena allocator.
	pub fn new(size: usize) -> Option<Self> {
		Some(Self {
			curr_offset: Cell::new(0),
			uncommitted_offset: Cell::new(0),
			vm: VirtualMemory::reserve(size)?,
		})
	}

	/// Allocate some region on the arena.
	pub fn alloc_region(&self, size: usize, align: usize) -> ArenaBox<'_, [u8]> {
		// get pointer to allocation, take alignment into account
		let aligned_offset = os::align_to(self.curr_offset.get(), align);
		let next_offset = aligned_offset + size;

		// commit pages we don't have yet
		if next_offset >= self.uncommitted_offset.get() {
			let next_uncommitted_offset = os::align_to(next_offset, os::page_size());

			let commit_offset = self.uncommitted_offset.get();
			let commit_size = next_uncommitted_offset - commit_offset;

			let commit_success = unsafe { self.vm.commit_unchecked(commit_offset, commit_size) };

			if !commit_success {
				panic!("OUT OF MEMORY D:");
			}

			self.uncommitted_offset.set(next_uncommitted_offset);
		}

		let ptr = unsafe { self.vm.addr_at(aligned_offset) };

		self.curr_offset.set(next_offset);

		ArenaBox {
			ptr: unsafe { NonNull::new_unchecked(slice_from_raw_parts_mut(ptr.as_ptr(), size)) },
			_phantom: PhantomData,
		}
	}

	pub fn alloc<T: 'static>(&self, value: T) -> ArenaBox<'_, T> {
		// A disadvantage of Rust as it stands:
		// T needs to be on the stack first instead of being initialized directly on the heap.
		// Simply returning uninitialized memory would be error-prone if T happens to change.
		// With a different language design, I think that trade-off wouldn't exist.
		// I could've had some unsafe API with MaybeUninit stuff but I didn't bother.

		let abox = self.alloc_region(size_of::<T>(), align_of::<T>());

		let ptr = abox.ptr.cast::<T>();
		unsafe { ptr.as_ptr().write(value) };

		ArenaBox {
			ptr,
			_phantom: PhantomData,
		}
	}

	#[inline]
	pub fn checkpoint(&self, f: impl FnOnce(&Self)) {
		let checkpoint = self.curr_offset.get();

		f(self);

		self.curr_offset.set(checkpoint);
	}

	// pub fn checkpoint(&self) -> ArenaCheckpoint<'_> {
	// 	ArenaCheckpoint {
	// 		offset: self.curr_offset.get(),
	// 		_phantom: PhantomData,
	// 	}
	// }

	// pub fn restore(&self, checkpoint: ArenaCheckpoint<'_>) {
	// 	self.curr_offset.set(checkpoint.offset);
	// }

	pub fn free_all(&mut self) {
		// all arena refs become invalid after that because of exclusive borrow
		self.curr_offset.set(0);
	}
}

/// An owned resource that comes from an arena allocator.
pub struct ArenaBox<'a, T: ?Sized> {
	ptr: NonNull<T>,
	_phantom: PhantomData<&'a T>,
}

impl<'a, T> AsMut<T> for ArenaBox<'a, T> {
	fn as_mut(&mut self) -> &mut T {
		unsafe { self.ptr.as_mut() }
	}
}

impl<'a, T> AsRef<T> for ArenaBox<'a, T> {
	fn as_ref(&self) -> &T {
		unsafe { self.ptr.as_ref() }
	}
}

pub struct ArenaCheckpoint<'a> {
	offset: usize,
	_phantom: PhantomData<&'a ()>,
}
