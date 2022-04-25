#![feature(once_cell)]

pub mod codegen;
pub mod memedit;
pub mod params;
pub mod pointers;
pub mod version;

pub mod prelude {
    pub use crate::codegen::*;
    pub use crate::memedit::*;
    pub use crate::params::*;
    pub use crate::pointers::*;
    pub use crate::version::*;

    pub use crate::{wait_option, wait_option_thread, ParamStruct, ParamVisitor};
}

/// Wait for an option value to be valid. Repeatedly calls the provided function, sleeping
/// 500 ms in between calls, until it returns `Some(T)`.
pub fn wait_option<T, F: FnMut() -> Option<T>>(mut f: F) -> T {
    loop {
        if let Some(t) = f() {
            return t;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

/// Wait for an option value to be valid. Repeatedly calls the provided function, sleeping
/// 500 ms in between calls, until it returns `Some(T)`. The waiting happens in a separate
/// thread.
pub fn wait_option_thread<
    T,
    F: 'static + Send + FnMut() -> Option<T>,
    G: 'static + Send + FnMut(T),
>(
    mut f: F,
    mut g: G,
) {
    std::thread::spawn(move || loop {
        if let Some(t) = f() {
            g(t);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    });
}

/// Implemented by generated code. Each method allows visiting a param's value by name and
/// stores the visited value in the second argument.
pub trait ParamVisitor {
    fn visit_u8(&mut self, name: &str, v: &mut u8);
    fn visit_u16(&mut self, name: &str, v: &mut u16);
    fn visit_u32(&mut self, name: &str, v: &mut u32);
    fn visit_i8(&mut self, name: &str, v: &mut i8);
    fn visit_i16(&mut self, name: &str, v: &mut i16);
    fn visit_i32(&mut self, name: &str, v: &mut i32);
    fn visit_f32(&mut self, name: &str, v: &mut f32);
    fn visit_bool(&mut self, name: &str, v: &mut bool);
}

/// Implemented by generated code. Visits all the fields of a `ParamVisitor`.
pub trait ParamStruct {
    fn visit<T: ParamVisitor + ?Sized>(&mut self, t: &mut T);
}

pub fn print_hex<T: Sized>(ptr: *const T) {
    let ptr = ptr as *const u8;

    let bytes: Vec<u8> = (0..std::mem::size_of::<T>())
        .map(|i| unsafe { *ptr.add(i) })
        .collect();

    bytes.chunks(16).for_each(|bs| {
        for i in bs {
            print!("{:02x} ", i);
        }

        print!("  ");

        for _ in bs.len()..16 {
            print!("  ");
        }

        for i in bs {
            let c = *i as char;
            print!("{}", if c.is_ascii() { c } else { '.' });
        }

        println!();
    });
}
