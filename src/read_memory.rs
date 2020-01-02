use scopeguard::defer;
use winapi::{
	ctypes::*,
	um::{handleapi::CloseHandle, tlhelp32::*, winnt::HANDLE},
};

fn get_pid(process_name: &str) -> anyhow::Result<u32> {
	let all_processes_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
	defer!(if unsafe { CloseHandle(all_processes_snapshot) } == 0 {
		panic!("can't close handle");
	});

	let mut process = PROCESSENTRY32W::default();
	{
		#![allow(clippy::cast_possible_truncation)]
		process.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
	}

	if unsafe { Process32FirstW(all_processes_snapshot, &mut process) } == 0 {
		return Err(anyhow::anyhow!("can't enumerate processes"));
	}

	loop {
		let inspected_process_name = widestring::U16Str::from_slice(&process.szExeFile).to_string_lossy();
		if process_name == inspected_process_name {
			break Ok(process.th32ProcessID);
		}

		if unsafe { Process32NextW(all_processes_snapshot, &mut process) } == 0 {
			return Err(anyhow::anyhow!("can't find your process"));
		}
	}
}

pub fn process_path(process_name: &str) -> anyhow::Result<String> {
	let pid = get_pid(process_name)?;

	let process_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };
	defer!(if unsafe { CloseHandle(process_snapshot) } == 0 {
		panic!("can't close handle");
	});

	let mut module_entry = MODULEENTRY32W::default();
	{
		#![allow(clippy::cast_possible_truncation)]
		module_entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;
	}

	if unsafe { Module32FirstW(process_snapshot, &mut module_entry) } != 1 {
		return Err(anyhow::anyhow!("can't enumerate modules"));
	}

	Ok(widestring::U16Str::from_slice(&module_entry.szExePath).to_string_lossy())
}

#[derive(Debug)]
pub struct Module {
	pub pid: u32,
	pub base_address: usize,
	pub base_size: usize,
	snapshot: HANDLE,
}

impl Module {
	pub fn from_pid(pid: u32, dll: &str) -> anyhow::Result<Self> {
		let process_snapshot: HANDLE = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };

		let mut module_entry = MODULEENTRY32W::default();
		{
			#![allow(clippy::cast_possible_truncation)]
			module_entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;
		}

		if unsafe { Module32FirstW(process_snapshot, &mut module_entry) } != 1 {
			return Err(anyhow::anyhow!("can't enumerate modules"));
		}

		loop {
			let module_name = widestring::U16Str::from_slice(&module_entry.szModule).to_string_lossy();

			if module_name == dll {
				return Ok(Self {
					pid,
					base_address: module_entry.modBaseAddr as usize,
					snapshot: process_snapshot,
					base_size: module_entry.modBaseSize as usize,
				});
			}

			if unsafe { Module32NextW(process_snapshot, &mut module_entry) } == 0 {
				if unsafe { CloseHandle(process_snapshot) } == 0 {
					panic!("can't close handle");
				}
				break Err(anyhow::anyhow!("can't find your module"));
			}
		}
	}

	pub fn new(process_name: &str, dll: &str) -> anyhow::Result<Self> { Self::from_pid(get_pid(process_name)?, dll) }
}

impl Drop for Module {
	fn drop(&mut self) {
		if unsafe { CloseHandle(self.snapshot) } == 0 {
			panic!("can't close handle");
		}
	}
}

pub fn read_mem<T: Default + Copy + Sized>(pid: u32, address: usize) -> anyhow::Result<T> {
	let buffer: *mut T = &mut Default::default();
	unsafe {
		let success = Toolhelp32ReadProcessMemory(pid, address as *const c_void, buffer as *mut c_void, std::mem::size_of::<T>(), &mut 0);
		if success == 1 { Ok(*buffer) } else { Err(anyhow::anyhow!("can't read")) }
	}
}
