#![no_std]
#![no_main]
#![feature(lang_items)]
#![allow(internal_features)]
#![allow(dead_code)]

use core::mem::{align_of, size_of};

use alloc::arena::Arena;

mod alloc;
mod os;

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

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
struct ColorU8 {
	r: u8,
	g: u8,
	b: u8,
}

fn usize_to_str(buffer: &mut [u8], mut value: usize) -> &str {
	// https://tia.mat.br/posts/2014/06/23/integer_to_string_conversion.html
	// lwan implementation

	let mut idx = buffer.len() - 1;
	buffer[idx] = b'\0';

	loop {
		idx -= 1;
		buffer[idx] = b"0123456789"[value % 10];

		if value / 10 == 0 {
			break;
		}

		value /= 10;
	}

	unsafe { core::str::from_utf8_unchecked(&buffer[idx..buffer.len()]) }
}

fn display_usize(arena: &Arena, n: usize) {
	let mut buffer = arena.alloc_region(3 * size_of::<usize>(), align_of::<u8>());
	let buffer = buffer.as_mut();

	let text_n = usize_to_str(buffer, n);
	os::bad_print(text_n);
}

fn display_color(arena: &Arena, color: &ColorU8) {
	arena.checkpoint(|arena| {
		let mut buffer = arena.alloc_region(3 * size_of::<u32>(), align_of::<u8>());
		let buffer = buffer.as_mut();

		os::bad_print("ColorU8 (\0");

		let text_r = usize_to_str(buffer, color.r as usize);
		os::bad_print(text_r);
		os::bad_print(", \0");

		let text_g = usize_to_str(buffer, color.g as usize);
		os::bad_print(text_g);
		os::bad_print(", \0");

		let text_b = usize_to_str(buffer, color.b as usize);
		os::bad_print(text_b);
		os::bad_print(")\n\0");
	});
}

#[allow(unused)]
fn things() {
	os::bad_print("Arena allocator!\n\0");

	let mut arena = Arena::new(os::total_phys_ram()).unwrap();

	let red = arena.alloc(ColorU8 { r: 255, g: 0, b: 0 });
	let green = arena.alloc(ColorU8 { r: 0, g: 255, b: 0 });
	let blue = arena.alloc(ColorU8 { r: 0, g: 0, b: 255 });

	let yellow = arena.alloc(ColorU8 { r: 255, g: 255, b: 0 });
	let cyan = arena.alloc(ColorU8 { r: 0, g: 255, b: 255 });
	let magenta = arena.alloc(ColorU8 { r: 255, g: 0, b: 255 });

	os::bad_print("\n\0");
	display_color(&arena, red.as_ref());
	display_color(&arena, green.as_ref());
	display_color(&arena, blue.as_ref());
	display_color(&arena, yellow.as_ref());
	display_color(&arena, cyan.as_ref());
	display_color(&arena, magenta.as_ref());

	const LEN: usize = 4096 * 1000;

	os::bad_print("\nAllocating \0");
	display_usize(&arena, LEN * size_of::<ColorU8>());
	os::bad_print(" bytes\n\0");

	let things = arena.alloc_fixed_slice::<_, LEN>(ColorU8 { r: 255, g: 255, b: 255 });

	os::bad_print("Displaying \0");
	display_usize(&arena, LEN);
	os::bad_print(" colors...\n\0");

	for thing in things.as_ref() {
		os::bad_print("  \0");
		display_color(&arena, thing);
	}
	os::bad_print(":p\n\0");

	arena.free_all();
}
