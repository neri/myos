// Concurrent Ring Buffer

use crate::myos::arch::cpu::Cpu;
use alloc::boxed::Box;
use alloc::vec::*;
use core::intrinsics::*;
use core::sync::atomic::*;

pub struct ConcurrentRingBuffer<T>
where
    T: Sized + Clone + Copy + Sync + Send,
{
    read: AtomicUsize,
    write: AtomicUsize,
    free: AtomicUsize,
    count: AtomicUsize,
    mask: usize,
    // buf: Vec<T>,
    buf: Box<[T]>,
}

unsafe impl<T> Sync for ConcurrentRingBuffer<T> where T: Sized + Clone + Copy + Sync + Send {}

impl<T> ConcurrentRingBuffer<T>
where
    T: Sized + Clone + Copy + Sync + Send,
{
    pub fn with_capacity(capacity: usize) -> Box<Self> {
        assert_eq!(capacity.count_ones(), 1);
        let mask = capacity - 1;
        let mut buf = Vec::<T>::with_capacity(capacity);
        unsafe {
            buf.set_len(capacity);
        }
        let buf = buf.into_boxed_slice();
        Box::new(Self {
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
            free: AtomicUsize::new(mask),
            mask: mask,
            buf: buf,
        })
    }

    pub fn read(&mut self) -> Option<T> {
        let mut count = self.count.load(Ordering::Relaxed);
        while count > 0 {
            match self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let read = self.read.fetch_add(1, Ordering::SeqCst);
                    let result: T;
                    unsafe {
                        let ptr = self.buf.as_ptr().add(read & self.mask);
                        result = ptr.read_volatile();
                    }
                    self.free.fetch_add(1, Ordering::SeqCst);
                    return Some(result);
                }
                Err(x) => {
                    count = x;
                    Cpu::relax();
                }
            }
        }
        None
    }

    pub fn write(&mut self, data: T) -> Result<(), ()> {
        let mut free = self.free.load(Ordering::Relaxed);
        while free > 0 {
            match self.free.compare_exchange_weak(
                free,
                free - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let write = self.write.fetch_add(1, Ordering::SeqCst);
                    unsafe {
                        let ptr = self.buf.as_mut_ptr().add(write & self.mask);
                        ptr.write_volatile(data);
                    }
                    self.count.fetch_add(1, Ordering::SeqCst);
                    return Ok(());
                }
                Err(x) => {
                    free = x;
                    Cpu::relax();
                }
            }
        }
        Err(())
    }
}