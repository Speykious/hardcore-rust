use core::arch::asm;
use core::ptr::NonNull;

use libc::{MAP_ANON, MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE};

/// Exit the program.
pub fn exit(code: i32) -> ! {
	let syscall_number: u64 = 60;

	unsafe {
		asm!(
			"syscall",
			in("rax") syscall_number,
			in("rdi") code,
			options(noreturn)
		)
	}
}

/// If the text is empty or not null-terminated, it won't print anything.
pub fn bad_print(text: &str) {
	if !text.as_bytes().last().is_some_and(|&c| c == b'\0') {
		return;
	}

	unsafe { libc::printf(text.as_ptr() as *const _) };
}

/// Total amount of physical RAM available on this system.
pub fn total_phys_ram() -> usize {
	phys_pages_count() * page_size()
}

/// Number of physical memory pages available on this system.
pub fn phys_pages_count() -> usize {
	unsafe { libc::sysconf(libc::_SC_PHYS_PAGES) as usize }
}

/// Size of memory pages on this system.
pub fn page_size() -> usize {
	unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize }
}

/// Maximum amount of virtual memory we can request.
pub fn addr_space_limit() -> usize {
	let mut addr_space_limit = libc::rlimit {
		rlim_cur: 0,
		rlim_max: 0,
	};

	unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut addr_space_limit) };

	addr_space_limit.rlim_cur as usize
}

/// A slice of virtual memory.
pub struct VirtualMemory {
	addr: NonNull<u8>,
	size: usize,
}

impl VirtualMemory {
	/// Reserve `size` bytes of virtual memory.
	///
	/// The virtual memory will be released once it drops.
	pub fn reserve(size: usize) -> Option<Self> {
		let addr = unsafe { libc::mmap(core::ptr::null_mut(), size, PROT_NONE, MAP_PRIVATE | MAP_ANON, -1, 0) };

		if addr as usize == usize::MAX {
			// invalid address
			return None;
		}

		Some(Self {
			addr: NonNull::new(addr as *mut u8)?,
			size,
		})
	}

	/// Commit a slice of memory.
	///
	/// The address and size get automatically page-aligned.
	pub fn commit(&self, offset: usize, size: usize) -> bool {
		let page_size = page_size();
		let size_aligned = align_to(size, page_size);
		let offset_aligned = align_to(offset, page_size);
		unsafe { self.commit_unchecked(offset_aligned, size_aligned) }
	}

	/// Uncommit a slice of memory.
	///
	/// The address and size get automatically page-aligned.
	pub fn uncommit(&self, offset: usize, size: usize) -> bool {
		let page_size = page_size();
		let size_aligned = align_to(size, page_size);
		let offset_aligned = align_to(offset, page_size);
		unsafe { self.uncommit_unchecked(offset_aligned, size_aligned) }
	}

	/// Commit a slice of memory.
	///
	/// # Safety
	///
	/// The address and size should be page-aligned manually.
	pub unsafe fn commit_unchecked(&self, offset_aligned: usize, size_aligned: usize) -> bool {
		libc::mprotect(
			self.addr.as_ptr().add(offset_aligned) as *mut _,
			size_aligned,
			PROT_READ | PROT_WRITE,
		) == 0
	}

	/// Uncommit a slice of memory.
	///
	/// # Safety
	///
	/// The address and size should be page-aligned manually.
	pub unsafe fn uncommit_unchecked(&self, offset_aligned: usize, size_aligned: usize) -> bool {
		libc::mprotect(
			self.addr.as_ptr().add(offset_aligned) as *mut _,
			size_aligned,
			PROT_NONE,
		) == 0
	}

	pub unsafe fn addr_at(&self, offset: usize) -> NonNull<u8> {
		// No need to check if it's non-null since it comes from non-null
		NonNull::new_unchecked(self.addr.as_ptr().add(offset))
	}
}

impl Drop for VirtualMemory {
	fn drop(&mut self) {
		unsafe { libc::munmap(self.addr.as_ptr() as *mut _, self.size) };
	}
}

#[inline]
pub const fn align_to(value: usize, to: usize) -> usize {
	let aligned = value + to - 1;
	aligned - aligned % to
}
