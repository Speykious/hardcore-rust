#![no_std]
#![no_main]
#![feature(lang_items)]
#![allow(internal_features)]
#![allow(dead_code)]

use alloc::arena::Arena;

mod alloc;
mod os;

extern crate libc;

#[cfg(not(test))]
#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
	os::bad_print("Panic D:\n\0");
	os::exit(1)
}

// These functions are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
	things();

	0
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct ColorU8 {
	r: u8,
	g: u8,
	b: u8,
}

fn do_something_with_color(_color: &ColorU8) {
	os::bad_print("doing something with some color idk\n\0");
}

#[allow(unused)]
fn things() {
	os::bad_print("Arena allocator!\n\0");

	let mut arena = Arena::new(1024).unwrap();

	let red = arena.alloc(ColorU8 { r: 255, g: 0, b: 0 });
	let green = arena.alloc(ColorU8 { r: 0, g: 255, b: 0 });
	let blue = arena.alloc(ColorU8 { r: 0, g: 0, b: 255 });

	let yellow = arena.alloc(ColorU8 { r: 255, g: 255, b: 0 });
	let cyan = arena.alloc(ColorU8 { r: 0, g: 255, b: 255 });
	let magenta = arena.alloc(ColorU8 { r: 255, g: 0, b: 255 });

	do_something_with_color(blue.as_ref());
	arena.free_all();
}
