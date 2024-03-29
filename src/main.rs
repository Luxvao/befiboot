#![no_std]
#![no_main]

extern crate alloc;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use log::info;
use uefi::{
    fs::{FileSystem, Path, PathBuf},
    prelude::*, table::Runtime,
};
use uefi_services::init;
use xmas_elf::ElfFile;

#[entry]
unsafe fn boot_main(efi_handle: Handle, mut table: SystemTable<Boot>) -> Status {
    match init(&mut table) {
        Ok(_) => (),
        Err(_) => exit_efi(efi_handle, &table),
    }

    let mut vars = parse_config(&table, &efi_handle).expect("Failed to parse config!");

    
    let esp = vars.get(String::from("esp")).unwrap().clone();

    let kernel = vars.get(String::from("kernel")).unwrap().clone();

    info!("ESP: {}", esp);

    info!("Kernel: {}", kernel);

    let mut kernel_path = PathBuf::new();

    load_kernel(&table, kernel_path, &efi_handle).expect("Unable to load the kernel!");

    Status::SUCCESS
}

fn parse_config(table: &SystemTable<Boot>, efi_handle: &Handle) -> Result<HashMap<String, String>, ()> {
    info!("Getting disk...");

    let simple_disk;

    unsafe {
        simple_disk = match table.boot_services().get_image_file_system(*efi_handle) {
            Ok(disk) => disk,
            Err(_) => exit_efi(*efi_handle, table),
        };
    }

    let mut disk = FileSystem::new(simple_disk);

    info!("DONE.");

    info!("Reading config...");

    let config_path = Path::new(cstr16!("boot\\befiboot\\config.conf"));

    let config;

    unsafe {
        config = match disk.read_to_string(config_path) {
            Ok(config) => config,
            Err(_) => exit_efi(*efi_handle, &table),
        };
    }

    info!("DONE.");

    info!("Parsing config...");

    let mut tokens = config
        .split(|c| c == ' ' || c == '\n')
        .map(|i| i.trim())
        .filter(|i| !i.eq(&""));

    let mut vars: HashMap<String, String> = HashMap::new();

    while let Some(item) = tokens.next() {
        let key = item;

        let eq; 

        unsafe {
            eq = match tokens.next() {
                Some(eq) => eq,
                None => {
                    log::error!("Syntax error! Expected '=' !");
                    exit_efi(*efi_handle, &table);
                }
            };
        }

        let val;

        unsafe {
            val = match tokens.next() {
                Some(val) => val,
                None => {
                    log::error!("Syntax error! Expected value!");
                    exit_efi(*efi_handle, &table);
                }
            };
        }

        if eq.trim() != "=" {
            log::error!("Syntax error! Expected '=' !");
            unsafe {
                exit_efi(*efi_handle, &table);
            }
        }

        vars.insert(key.to_owned(), val.to_owned());
    }

    info!("DONE.");

    Ok(vars)
}

fn load_kernel(table: &SystemTable<Boot>, kernel_path: PathBuf, handle: &Handle) -> Result<*const fn(&SystemTable<Runtime>), ()> {

    let mut kernel_entry: *const fn(&SystemTable<Runtime>) = core::ptr::null();

    let simple_fs = match table.boot_services().get_image_file_system(*handle) {
        Ok(fs) => fs,
        Err(_) => unsafe {
            exit_efi(*handle, table);
        }
    };

    let mut fs = FileSystem::new(simple_fs);

    let kernel_raw = fs.read(kernel_path).expect("Failed to read kernel from disk");

    let kernel = ElfFile::new(&kernel_raw).expect("Unable to parse kernel");

    Ok(kernel_entry)
}

unsafe fn exit_efi(handle: Handle, table: &SystemTable<Boot>) -> ! {
    table
        .boot_services()
        .exit(handle, Status::ABORTED, 0, core::ptr::null_mut());
}

#[derive(Debug, Clone, PartialEq)]
struct HashMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

#[allow(dead_code)]
impl<K, V> HashMap<K, V>
where
    K: PartialEq,
{
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    fn insert(&mut self, key: K, val: V) {
        self.keys.push(key);

        self.values.push(val);
    }

    fn remove(&mut self, key: K) -> Result<(), ()> {
        let position = match self.keys.iter().position(|i| i.eq(&key)) {
            Some(pos) => pos,
            None => return Err(()),
        };

        self.keys.remove(position);

        self.values.remove(position);

        Ok(())
    }

    fn get(&mut self, key: K) -> Option<&V> {
        let position = match self.keys.iter().position(|i| i.eq(&key)) {
            Some(pos) => pos,
            None => return None,
        };

        self.values.get(position).into()
    }
}
