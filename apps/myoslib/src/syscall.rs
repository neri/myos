// myos system calls

#[link(wasm_import_module = "arl")]
extern "C" {
    pub fn svc0(_: usize) -> usize;
    pub fn svc1(_: usize, _: usize) -> usize;
    pub fn svc2(_: usize, _: usize, _: usize) -> usize;
    pub fn svc3(_: usize, _: usize, _: usize, _: usize) -> usize;
    pub fn svc4(_: usize, _: usize, _: usize, _: usize, _: usize) -> usize;
    pub fn svc5(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize) -> usize;
    pub fn svc6(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize) -> usize;
}

/// Display a string
#[inline]
pub fn os_print(s: &str) {
    unsafe {
        svc2(1, s.as_ptr() as usize, s.len());
    }
}

/// Create a new window
#[inline]
pub fn os_new_window(s: &str, width: usize, height: usize) -> usize {
    unsafe { svc4(3, s.as_ptr() as usize, s.len(), width, height) }
}

/// Draw a string in a window
#[inline]
pub fn os_draw_text(window: usize, x: usize, y: usize, s: &str, color: u32) {
    let ptr = s.as_ptr() as usize;
    let len = s.len();
    unsafe {
        svc6(4, window, x, y, ptr, len, color as usize);
    }
}

/// Fill a rectangle in a window
#[inline]
pub fn os_fill_rect(window: usize, x: usize, y: usize, width: usize, height: usize, color: u32) {
    unsafe {
        svc6(5, window, x, y, width, height, color as usize);
    }
}

/// Wait for key input
#[inline]
pub fn os_wait_key(window: usize) -> u32 {
    unsafe { svc1(6, window) as u32 }
}

/// Draw a bitmap in a window
#[inline]
pub fn os_blt8(window: usize, x: usize, y: usize, bitmap: usize) {
    unsafe {
        svc4(7, window, x, y, bitmap);
    }
}

/// Draw a bitmap in a window
#[inline]
pub fn os_blt1(window: usize, x: usize, y: usize, bitmap: usize, color: u32, scale: isize) {
    unsafe {
        svc6(8, window, x, y, bitmap, color as usize, scale as usize);
    }
}

/// Reflect the window's bitmap to the screen now.
#[inline]
pub fn os_flip(window: usize) {
    unsafe {
        svc1(10, window);
    }
}

/// Get the value of the monotonic timer in microseconds
#[inline]
pub fn os_monotonic() -> u32 {
    unsafe { svc0(50) as u32 }
}

/// Return a random number
#[inline]
pub fn os_rand() -> u32 {
    unsafe { svc0(51) as u32 }
}

#[inline]
pub fn os_alloc(size: usize, align: usize) -> usize {
    unsafe { svc2(100, size, align) }
}

#[inline]
pub fn os_free(ptr: usize) {
    unsafe {
        svc1(101, ptr);
    }
}
