// My UEFI-Rust Playground
#![feature(abi_efiapi)]
#![feature(llvm_asm)]
#![no_std]
#![no_main]
// use aml;
use core::fmt::Write;
use uefi::prelude::*;
use uefi_pg::myos::arch::cpu::Cpu;
use uefi_pg::myos::bus::lpc;
use uefi_pg::myos::io::graphics::*;
use uefi_pg::myos::io::hid;
use uefi_pg::myos::thread::*;
use uefi_pg::*;

uefi_pg_entry!(main);

fn main(handle: Handle, st: SystemTable<Boot>) -> Status {
    let rsdptr = match st.find_config_table(uefi::table::cfg::ACPI2_GUID) {
        Some(val) => val,
        None => {
            writeln!(st.stdout(), "Error: ACPI Table Not Found").unwrap();
            return Status::LOAD_ERROR;
        }
    };

    // TODO: init custom allocator
    let buf_size = 0x1000000;
    let buf_ptr = st
        .boot_services()
        .allocate_pool(uefi::table::boot::MemoryType::LOADER_DATA, buf_size)
        .unwrap()
        .unwrap();
    myos::mem::alloc::init(buf_ptr as usize, buf_size);

    //////// GUARD //////// exit_boot_services //////// GUARD ////////
    let (_st, mm) = exit_boot_services(st, handle);

    let fb = stdout().fb();
    fb.reset();
    let size = fb.size();
    let center = Size::<isize>::new(size.width / 2, size.height / 2);

    fb.fill_rect(
        Rect::new(center.width - 80, center.height - 75, 50, 100),
        IndexedColor::LightRed.as_color(),
    );
    fb.fill_rect(
        Rect::new(center.width - 25, center.height - 75, 50, 100),
        IndexedColor::LightGreen.as_color(),
    );
    fb.fill_rect(
        Rect::new(center.width + 30, center.height - 75, 50, 100),
        IndexedColor::LightBlue.as_color(),
    );

    let mut total_memory_size: u64 = 0;
    for mem_desc in mm {
        if mem_desc.ty.is_countable() {
            total_memory_size += mem_desc.page_count << 12;
        }
    }
    unsafe {
        myos::arch::system::System::init(rsdptr as usize, total_memory_size, first_child);
    }
}

fn first_child(system: &myos::arch::system::System) {
    println!(
        "\nMy practice OS version {} Total {} Cores, {} MB Memory",
        myos::MyOs::version(),
        system.number_of_active_cpus(),
        system.total_memory_size() >> 20,
    );
    println!("Hello, {:#}!", "Rust");

    for i in 0..system.number_of_active_cpus() {
        let cpu = system.cpu(i);
        println!("CPU {} apic_id:{}", i, cpu.cpu_id.0);
    }

    Thread::spawn(|| {
        println!("Hello, thread!");
    });

    loop {
        match lpc::get_key() {
            Some((usage, modifier)) => {
                if usage != hid::Usage::NULL {
                    let c = hid::HidManager::usage_to_char_109(usage, modifier);
                    print!("{}", c);
                    if c == 'p' {
                        myos::scheduler::GlobalScheduler::print_statistics();
                    }
                }
            }
            None => unsafe { Cpu::halt() },
        }
    }
}
