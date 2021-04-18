// A Computer System

use crate::io::emcon::*;
use crate::io::tty::Tty;
use crate::task::scheduler::*;
use crate::*;
use crate::{arch::cpu::*, fonts::*};
use alloc::boxed::Box;
use alloc::string::*;
use alloc::vec::Vec;
use bootprot::BootInfo;
use core::fmt;
use core::ptr::*;
use megstd::drawing::*;
use megstd::time::SystemTime;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    versions: u32,
    rel: &'static str,
}

impl Version {
    const SYSTEM_NAME: &'static str = "codename MYOS";
    const SYSTEM_SHORT_NAME: &'static str = "myos";
    const RELEASE: &'static str = "";
    const VERSION: Version = Version::new(0, 0, 1, Self::RELEASE);

    const fn new(maj: u8, min: u8, patch: u16, rel: &'static str) -> Self {
        let versions = ((maj as u32) << 24) | ((min as u32) << 16) | (patch as u32);
        Version { versions, rel }
    }

    pub const fn as_u32(&self) -> u32 {
        self.versions
    }

    pub const fn maj(&self) -> usize {
        ((self.versions >> 24) & 0xFF) as usize
    }

    pub const fn min(&self) -> usize {
        ((self.versions >> 16) & 0xFF) as usize
    }

    pub const fn patch(&self) -> usize {
        (self.versions & 0xFFFF) as usize
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.rel.len() > 0 {
            write!(
                f,
                "{}.{}.{}-{}",
                self.maj(),
                self.min(),
                self.patch(),
                self.rel
            )
        } else {
            write!(f, "{}.{}.{}", self.maj(), self.min(), self.patch())
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct ProcessorId(pub u8);

impl ProcessorId {
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }
}

impl From<u8> for ProcessorId {
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl From<u32> for ProcessorId {
    fn from(val: u32) -> Self {
        Self(val as u8)
    }
}

impl From<usize> for ProcessorId {
    fn from(val: usize) -> Self {
        Self(val as u8)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ProcessorIndex(pub usize);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessorCoreType {
    Main,
    Sub,
}

#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct PhysicalAddress(pub usize);

pub struct System {
    /// Number of cpu cores
    num_of_cpus: usize,
    /// Number of physical cpu cores
    num_of_performance_cpus: usize,
    /// Vector of cpu cores
    cpus: Vec<Box<Cpu>>,

    /// An instance of ACPI tables
    acpi: Option<Box<acpi::AcpiTables<MyAcpiHandler>>>,

    /// An instance of SMBIOS
    smbios: Option<Box<fw::smbios::SMBIOS>>,

    /// Machine's manufacture name
    manufacturer: Option<String>,
    /// Machine's product name
    product: Option<String>,

    // screens
    main_screen: Option<Bitmap32<'static>>,
    em_console: EmConsole,
    stdout: Option<Box<dyn Tty>>,

    // copy of boot info
    boot_flags: BootFlags,
    initrd_base: usize,
    initrd_size: usize,
}

static mut SYSTEM: System = System::new();

impl System {
    const fn new() -> Self {
        System {
            num_of_cpus: 0,
            num_of_performance_cpus: 1,
            cpus: Vec::new(),
            acpi: None,
            smbios: None,
            manufacturer: None,
            product: None,
            boot_flags: BootFlags::empty(),
            main_screen: None,
            em_console: EmConsole::new(FontManager::fixed_system_font()),
            stdout: None,
            initrd_base: 0,
            initrd_size: 0,
        }
    }

    /// Init the system
    pub unsafe fn init(info: &BootInfo, f: fn() -> ()) -> ! {
        let shared = &mut SYSTEM;
        shared.boot_flags = info.flags;
        shared.initrd_base = info.initrd_base as usize;
        shared.initrd_size = info.initrd_size as usize;

        mem::MemoryManager::init_first(info);
        arch::page::PageManager::init(info);

        shared.main_screen = Some(Bitmap32::from_static(
            info.vram_base as usize as *mut TrueColor,
            Size::new(info.screen_width as isize, info.screen_height as isize),
            info.vram_stride as usize,
        ));

        shared.acpi = Some(Box::new(
            acpi::AcpiTables::from_rsdp(MyAcpiHandler::new(), info.acpi_rsdptr as usize).unwrap(),
        ));

        if info.smbios != 0 {
            let smbios = fw::smbios::SMBIOS::init(info.smbios as usize);
            let h = smbios.find(fw::smbios::HeaderType::SYSTEM_INFO).unwrap();
            let slice = h.as_slice();
            shared.manufacturer = h.string(slice[4] as usize).map(|v| v.to_string());
            shared.product = h.string(slice[5] as usize).map(|v| v.to_string());
            shared.smbios = Some(smbios);
        }

        let pi = Self::acpi_platform().processor_info.unwrap();
        shared.num_of_cpus = pi.application_processors.len() + 1;
        shared.cpus.reserve(shared.num_of_cpus);
        shared
            .cpus
            .push(Cpu::new(ProcessorId(pi.boot_processor.local_apic_id)));

        arch::Arch::init();

        bus::pci::Pci::init();

        Scheduler::start(Self::late_init, f as usize);
    }

    fn late_init(args: usize) {
        let shared = Self::shared();
        unsafe {
            mem::MemoryManager::late_init();

            fs::FileManager::init(shared.initrd_base, shared.initrd_size);

            rt::RuntimeEnvironment::init();

            if let Some(main_screen) = shared.main_screen.as_mut() {
                fonts::FontManager::init();
                window::WindowManager::init(main_screen.clone());
            }

            io::hid::HidManager::init();

            arch::Arch::late_init();

            user::userenv::UserEnv::start(core::mem::transmute(args));
        }
    }

    /// Returns an internal shared instance
    #[inline]
    fn shared() -> &'static mut System {
        unsafe { &mut SYSTEM }
    }

    /// Returns the name of current system.
    #[inline]
    pub const fn name() -> &'static str {
        &Version::SYSTEM_NAME
    }

    /// Returns abbreviated name of current system.
    #[inline]
    pub const fn short_name() -> &'static str {
        &Version::SYSTEM_SHORT_NAME
    }

    /// Returns the version of current system.
    #[inline]
    pub const fn version() -> &'static Version {
        &Version::VERSION
    }

    /// Returns the current system time.
    #[inline]
    pub fn system_time() -> SystemTime {
        arch::Arch::system_time()
    }

    /// Returns whether the kernel is multiprocessor-capable.
    #[inline]
    pub const fn is_multi_processor_capable_kernel() -> bool {
        true
    }

    /// Returns the number of logical CPU cores.
    #[inline]
    pub fn num_of_cpus() -> usize {
        Self::shared().num_of_cpus
    }

    /// Returns the number of performance CPU cores.
    /// Returns less than `num_of_cpus` for SMT-enabled processors or heterogeneous computing.
    #[inline]
    pub fn num_of_performance_cpus() -> usize {
        Self::shared().num_of_performance_cpus
    }

    /// Returns the number of active logical CPU cores.
    /// Returns the same value as `num_of_cpus` except during SMP initialization.
    #[inline]
    pub fn num_of_active_cpus() -> usize {
        Self::shared().cpus.len()
    }

    /// Add SMP-initialized CPU cores to the list of enabled cores.
    ///
    /// SAFETY: THREAD UNSAFE. DO NOT CALL IT EXCEPT FOR SMP INITIALIZATION.
    #[inline]
    pub(crate) unsafe fn activate_cpu(new_cpu: Box<Cpu>) {
        let shared = Self::shared();
        if new_cpu.processor_type() == ProcessorCoreType::Main {
            shared.num_of_performance_cpus += 1;
        }
        shared.cpus.push(new_cpu);
    }

    #[inline]
    pub fn cpu<'a>(index: usize) -> &'a Box<Cpu> {
        &Self::shared().cpus[index]
    }

    #[inline]
    pub(crate) unsafe fn cpu_mut<'a>(index: usize) -> &'a mut Box<Cpu> {
        &mut Self::shared().cpus[index]
    }

    /// SAFETY: THREAD UNSAFE. DO NOT CALL IT EXCEPT FOR SMP INITIALIZATION.
    #[inline]
    pub(crate) unsafe fn sort_cpus<F>(compare: F)
    where
        F: FnMut(&Box<Cpu>, &Box<Cpu>) -> core::cmp::Ordering,
    {
        Self::shared().cpus.sort_by(compare);
        let mut i = 0;
        for cpu in &mut Self::shared().cpus {
            cpu.cpu_index = ProcessorIndex(i);
            i += 1;
        }
    }

    #[inline]
    #[track_caller]
    pub fn acpi() -> &'static acpi::AcpiTables<MyAcpiHandler> {
        Self::shared().acpi.as_ref().unwrap()
    }

    #[inline]
    pub fn smbios() -> Option<&'static Box<fw::smbios::SMBIOS>> {
        Self::shared().smbios.as_ref()
    }

    #[inline]
    pub fn manufacturer<'a>() -> Option<&'a str> {
        Self::shared().manufacturer.as_ref().map(|v| v.as_str())
    }

    #[inline]
    pub fn product<'a>() -> Option<&'a str> {
        Self::shared().product.as_ref().map(|v| v.as_str())
    }

    #[inline]
    #[track_caller]
    pub fn acpi_platform() -> acpi::PlatformInfo {
        Self::acpi().platform_info().unwrap()
    }

    /// SAFETY: IT DESTROYS EVERYTHING.
    pub unsafe fn reset() -> ! {
        Cpu::reset();
    }

    /// SAFETY: IT DESTROYS EVERYTHING.
    pub unsafe fn shutdown() -> ! {
        todo!();
    }

    /// Get main screen
    pub fn main_screen() -> Bitmap<'static> {
        let shared = Self::shared();
        shared.main_screen.as_mut().unwrap().into()
    }

    pub fn em_console<'a>() -> &'a mut EmConsole {
        let shared = Self::shared();
        &mut shared.em_console
    }

    pub fn set_stdout(stdout: Box<dyn Tty>) {
        let shared = Self::shared();
        shared.stdout = Some(stdout);
    }

    pub fn stdout<'a>() -> &'a mut Box<dyn Tty> {
        let shared = Self::shared();
        shared.stdout.as_mut().unwrap()
    }
}

// pub struct DeviceInfo {
//     manufacturer: String,
//     product: String,
//     num_of_active_cpus: usize,
//     num_of_performance_cpus: usize,
// }

//-//-//-//-//

#[derive(Clone)]
pub struct MyAcpiHandler {}

impl MyAcpiHandler {
    const fn new() -> Self {
        MyAcpiHandler {}
    }
}

use acpi::PhysicalMapping;
impl ::acpi::AcpiHandler for MyAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(physical_address as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
            handler: Self::new(),
        }
    }
    fn unmap_physical_region<T>(&self, _region: &PhysicalMapping<Self, T>) {}
}
