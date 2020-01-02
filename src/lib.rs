#![feature(try_blocks)]

mod read_memory;

use anyhow::Context;
pub use read_memory::*;
use std::{
	ffi::{CStr, CString},
	os::raw::c_char,
};

#[repr(C)]
#[derive(Debug)]
pub struct FFIResult<T> {
	pub value: T,
	pub success: bool,
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn free_str(ptr: *mut c_char) { let _ = CString::from_raw(ptr); }

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[must_use]
pub unsafe extern "C" fn read_i32(
	process_name_raw: *const c_char,
	module_name_raw: *const c_char,
	offsets_ptr: *const usize,
	offsets_len: usize,
) -> FFIResult<i32> {
	let res: anyhow::Result<_> = try {
		let process_name = CStr::from_ptr(process_name_raw).to_str()?;
		let module_name = CStr::from_ptr(module_name_raw).to_str()?;
		let offsets = std::slice::from_raw_parts(offsets_ptr, offsets_len);
		let module = Module::new(process_name, module_name)?;

		// TODO: maybe nicer?
		let (first_offset, tail): (&usize, &[usize]) = offsets.split_first().context("no first offset")?;
		let (last_offset, middle) = tail.split_last().context("no last offset")?;
		let mut ptr = read_mem::<u32>(module.pid, first_offset + module.base_address)? as usize;
		for offset in middle {
			ptr = read_mem::<u32>(module.pid, offset + ptr)? as usize;
		}

		read_mem::<i32>(module.pid, last_offset + ptr)?
	};
	match res {
		Ok(value) => FFIResult { success: true, value },
		_ => FFIResult { success: false, value: 0 },
	}
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[must_use]
pub unsafe extern "C" fn process_path(process_name_raw: *const c_char) -> FFIResult<*mut c_char> {
	let res: anyhow::Result<_> = try {
		let process_name = CStr::from_ptr(process_name_raw).to_str()?;
		let path = read_memory::process_path(process_name)?;
		CString::new(path)?.into_raw()
	};
	match res {
		Ok(value) => FFIResult { success: true, value },
		_ => FFIResult { success: false, value: std::ptr::null_mut() },
	}
}
