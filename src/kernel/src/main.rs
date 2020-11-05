// My OS Entry
// (c) 2020 Nerry
// License: MIT

#![no_std]
#![no_main]
#![feature(asm)]

// use acpi;
use alloc::vec::*;
use arch::cpu::*;
use bootprot::*;
use core::fmt::Write;
// use core::time::Duration;
use io::graphics::*;
use kernel::*;
use mem::memory::*;
use mem::string;
use system::*;
// use task::executor::Executor;
use task::scheduler::*;
use task::Task;
use uuid::*;
use window::*;
// use alloc::boxed::Box;
// use mem::string::*;
// use io::fonts::*;
// use core::sync::atomic::*;

extern crate alloc;
extern crate rlibc;

entry!(main);

fn main() {
    MyScheduler::spawn_async(Task::new(repl_main()));
    // MyScheduler::spawn_async(Task::new(test_task()));
    MyScheduler::perform_tasks();
}

async fn repl_main() {
    exec("ver");

    loop {
        print!("# ");
        if let Some(cmdline) = stdout().read_line_async(120).await {
            exec(&cmdline);
        }
    }
}

#[allow(dead_code)]
async fn test_task() {
    let window_size = Size::new(640, 480);
    let window = WindowBuilder::new("MyOS Paint")
        .size(window_size)
        .origin(Point::new(50, 50))
        .default_message_queue()
        .build();

    window.show();

    let canvas = Bitmap::new(
        window_size.width as usize,
        window_size.height as usize,
        false,
    );
    canvas.fill_rect(canvas.size().into(), Color::from_rgb(0xFFFFFF));
    // canvas.draw_rect(canvas.size().into(), Color::from_rgb(0xFF0000));

    let current_pen_radius = 1;
    let current_pen = Color::from(IndexedColor::Black);
    let mut is_drawing = false;
    let mut last_pen = Point::new(0, 0);

    while let Some(message) = window.get_message().await {
        match message {
            WindowMessage::Draw => {
                window
                    .draw(|bitmap| {
                        bitmap.blt(&canvas, Point::new(0, 0), bitmap.bounds(), BltOption::COPY);
                    })
                    .unwrap();
            }
            WindowMessage::Char(c) => match c {
                'c' => {
                    canvas.fill_rect(canvas.bounds(), Color::WHITE);
                    window.set_needs_display();
                }
                _ => (),
            },
            WindowMessage::MouseMove(e) => {
                if is_drawing {
                    let e_point = e.point();
                    last_pen.line_to(e_point, |point| {
                        canvas.fill_circle(point, current_pen_radius, current_pen);
                    });
                    last_pen = e_point;
                    window.set_needs_display();
                }
            }
            WindowMessage::MouseDown(e) => {
                let e_point = e.point();
                canvas.fill_circle(e_point, current_pen_radius, current_pen);
                last_pen = e_point;
                is_drawing = true;
                window.set_needs_display();
            }
            WindowMessage::MouseUp(_e) => {
                is_drawing = false;
            }
            WindowMessage::MouseLeave => {
                is_drawing = false;
            }
            _ => window.handle_default_message(message),
        }
    }
}

#[allow(dead_code)]
fn draw_cursor(bitmap: &Bitmap, point: Point<isize>, color: Color) {
    let size = 7;
    let size2 = size / 2;
    bitmap.draw_vline(Point::new(point.x, point.y - size2), size, color);
    bitmap.draw_hline(Point::new(point.x - size2, point.y), size, color);
}

fn exec(cmdline: &str) {
    if cmdline.len() == 0 {
        return;
    }
    let mut sb = string::StringBuffer::with_capacity(cmdline.len());
    let mut args = Vec::new();
    let mut phase = CmdLinePhase::LeadingSpace;
    sb.clear();
    for c in cmdline.chars() {
        match phase {
            CmdLinePhase::LeadingSpace => match c {
                ' ' => (),
                _ => {
                    sb.write_char(c).unwrap();
                    phase = CmdLinePhase::Token;
                }
            },
            CmdLinePhase::Token => match c {
                ' ' => {
                    args.push(sb.as_str());
                    phase = CmdLinePhase::LeadingSpace;
                    sb.split();
                }
                _ => {
                    sb.write_char(c).unwrap();
                }
            },
        }
    }
    if sb.len() > 0 {
        args.push(sb.as_str());
    }

    if args.len() > 0 {
        let cmd = args[0];
        match command(cmd) {
            Some(exec) => {
                exec(args.as_slice());
            }
            None => println!("Command not found: {}", cmd),
        }
    }
}

enum CmdLinePhase {
    LeadingSpace,
    Token,
}

fn command(cmd: &str) -> Option<&'static fn(&[&str]) -> isize> {
    for command in &COMMAND_TABLE {
        if command.0 == cmd {
            return Some(&command.1);
        }
    }
    None
}

const COMMAND_TABLE: [(&str, fn(&[&str]) -> isize, &str); 9] = [
    ("help", cmd_help, "Show Help"),
    ("cls", cmd_cls, "Clear screen"),
    ("ver", cmd_ver, "Display version"),
    ("sysctl", cmd_sysctl, "System Control"),
    ("lspci", cmd_lspci, "Show List of PCI Devices"),
    ("uuidgen", cmd_uuidgen, ""),
    ("reboot", cmd_reboot, "Restart computer"),
    ("exit", cmd_reserved, ""),
    ("echo", cmd_echo, ""),
];

fn cmd_reserved(_: &[&str]) -> isize {
    println!("Feature not available");
    1
}

fn cmd_reboot(_: &[&str]) -> isize {
    unsafe {
        System::reset();
    }
}

fn cmd_help(_: &[&str]) -> isize {
    for cmd in &COMMAND_TABLE {
        if cmd.2.len() > 0 {
            println!("{}\t{}", cmd.0, cmd.2);
        }
    }
    0
}

fn cmd_cls(_: &[&str]) -> isize {
    match stdout().reset() {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn cmd_ver(_: &[&str]) -> isize {
    println!("{} v{}", System::name(), System::version(),);
    0
}

fn cmd_echo(args: &[&str]) -> isize {
    println!("{}", args[1..].join(" "));
    0
}

fn cmd_uuidgen(_: &[&str]) -> isize {
    match Uuid::generate() {
        Some(v) => {
            println!("{}", v);
            return 0;
        }
        None => {
            println!("Feature not available");
            return 1;
        }
    }
}

fn cmd_sysctl(argv: &[&str]) -> isize {
    if argv.len() < 2 {
        println!("usage: sysctl command [options]");
        println!("memory:\tShow memory information");
        return 1;
    }
    let subcmd = argv[1];
    match subcmd {
        "memory" => {
            let mut sb = string::StringBuffer::with_capacity(256);
            MemoryManager::statistics(&mut sb);
            print!("{}", sb.as_str());
        }
        "random" => match Cpu::secure_rand() {
            Ok(rand) => println!("{:016x}", rand),
            Err(_) => println!("# No SecureRandom"),
        },
        "cpuid" => {
            let cpuid0 = Cpu::cpuid(0x000_0000, 0);
            let cpuid1 = Cpu::cpuid(0x000_0001, 0);
            let cpuid7 = Cpu::cpuid(0x000_0007, 0);
            let cpuid81 = Cpu::cpuid(0x8000_0001, 0);
            println!("CPUID {:08x}", cpuid0.eax());
            println!(
                "Feature 0~1 EDX {:08x} ECX {:08x}",
                cpuid1.edx(),
                cpuid1.ecx(),
            );
            println!(
                "Feature 0~7 EBX {:08x} ECX {:08x} EDX {:08x}",
                cpuid7.ebx(),
                cpuid7.ecx(),
                cpuid7.edx(),
            );
            println!(
                "Feature 8~1 EDX {:08x} ECX {:08x}",
                cpuid81.edx(),
                cpuid81.ecx(),
            );
        }
        _ => {
            println!("Unknown command: {}", subcmd);
            return 1;
        }
    }
    0
}

fn cmd_lspci(argv: &[&str]) -> isize {
    let opt_all = argv.len() > 1;
    for device in bus::pci::Pci::devices() {
        let addr = device.address();
        println!(
            "{:02x}.{:02x}.{} {:04x}:{:04x} {:06x} {}",
            addr.0,
            addr.1,
            addr.2,
            device.vendor_id().0,
            device.device_id().0,
            device.class_code(),
            device.class_string(),
        );
        if opt_all {
            for function in device.functions() {
                let addr = function.address();
                println!(
                    "     .{} {:04x}:{:04x} {:06x} {}",
                    addr.2,
                    function.vendor_id().0,
                    function.device_id().0,
                    function.class_code(),
                    function.class_string(),
                );
            }
        }
    }
    0
}
