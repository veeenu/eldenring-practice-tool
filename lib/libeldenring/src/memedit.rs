use std::fmt::{Debug, Display};
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::ptr::null_mut;

use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Threading::GetCurrentProcess;

/// Wraps CheatEngine's concept of pointer with nested offsets. Evaluates,
/// if the evaluation does not fail, to a mutable pointer of type `T`.
///
/// At runtime, it evaluates the final address of the chain by reading the
/// base pointer, then recursively reading the next memory address in the
/// chain at an offset from there. For example,
///
/// ```
/// PointerChain::<T>::new(&[a, b, c, d, e])
/// ```
///
/// evaluates to
///
/// ```
/// *(*(*(*(*a + b) + c) + d) + e)
/// ```
///
/// This is useful for managing reverse engineered structures which are not
/// fully known.
#[derive(Clone)]
pub struct PointerChain<T> {
    proc: HANDLE,
    base: *mut T,
    offsets: Vec<usize>,
}
unsafe impl<T> Send for PointerChain<T> {}
unsafe impl<T> Sync for PointerChain<T> {}

impl<T> PointerChain<T> {
    /// Creates a new pointer chain given an array of addresses.
    pub fn new(chain: &[usize]) -> PointerChain<T> {
        let mut it = chain.iter();
        let base = *it.next().unwrap() as usize as *mut T;
        PointerChain {
            proc: unsafe { GetCurrentProcess() },
            base,
            offsets: it.copied().collect(), // it.map(|x| *x).collect(),
        }
    }

    fn safe_read(&self, addr: usize, offs: usize) -> Option<usize> {
        let mut value = 0usize;
        let result = unsafe {
            ReadProcessMemory(
                self.proc,
                addr as _,
                &mut value as *mut usize as _,
                std::mem::size_of::<usize>(),
                null_mut(),
            )
        }
        .0;

        match result {
            0 => None,
            _ => Some(value + offs as usize),
        }
    }

    /// Safely evaluates the pointer chain.
    /// Relies on `ReadProcessMemory` instead of pointer dereferencing for crash
    /// safety.  Returns `None` if the evaluation failed.
    pub fn eval(&self) -> Option<*mut T> {
        self.offsets
            .iter()
            .fold(Some(self.base as usize), |addr, &offs| {
                addr.and_then(|addr| self.safe_read(addr, offs))
            })
            .map(|addr| addr as *mut T)
    }

    /// Evaluates the pointer chain and attempts to read the datum.
    /// Returns `None` if either the evaluation or the read failed.
    pub fn read(&self) -> Option<T> {
        if let Some(ptr) = self.eval() {
            let mut value: T = unsafe { std::mem::zeroed() };
            let result = unsafe {
                ReadProcessMemory(
                    self.proc,
                    ptr as _,
                    &mut value as *mut _ as _,
                    std::mem::size_of::<T>(),
                    null_mut(),
                )
            };

            match result {
                BOOL(0) => None,
                _ => Some(value),
            }
        } else {
            None
        }
    }

    /// Evaluates the pointer chain and attempts to write the datum.
    /// Returns `None` if either the evaluation or the write failed.
    pub fn write(&self, mut value: T) -> Option<()> {
        if let Some(ptr) = self.eval() {
            let result = unsafe {
                WriteProcessMemory(
                    self.proc,
                    ptr as _,
                    &mut value as *mut _ as _,
                    std::mem::size_of::<T>(),
                    null_mut(),
                )
            };

            match result {
                BOOL(0) => None,
                _ => Some(()),
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Bitflag<T>(PointerChain<T>, T);

impl<T: Display + Debug> Debug for PointerChain<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PointerChain({} @ {:p}", self.proc.0, self.base)?;
        for o in &self.offsets {
            write!(f, ", {:x}", o)?;
        }
        write!(f, ")")
    }
}

impl<T: Display + Debug> Debug for Bitflag<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bitflag(bit {} of {:?})", self.1, self.0)
    }
}

impl<T> Bitflag<T>
where
    T: BitXor<Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + Not<Output = T>
        + PartialEq
        + Copy,
{
    pub fn new(c: PointerChain<T>, mask: T) -> Self {
        Bitflag(c, mask)
    }

    pub fn toggle(&self) -> Option<bool> {
        if let Some(x) = self.0.read() {
            self.0.write(x ^ self.1);
            Some(x == self.1)
        } else {
            None
        }
    }

    pub fn get(&self) -> Option<bool> {
        self.0.read().map(|x| (x & self.1) == self.1)
    }

    pub fn set(&self, flag: bool) {
        if let Some(x) = self.0.read() {
            self.0.write(if flag { x | self.1 } else { x & !self.1 });
        }
    }
}

#[macro_export]
macro_rules! pointer_chain {
    ($($e:expr),+) => { PointerChain::new(&[$($e,)*]) }
}

#[macro_export]
macro_rules! bitflag {
    ($b:expr; $($e:expr),+) => { Bitflag::new(PointerChain::new(&[$($e,)*]), $b) }
}

pub use {bitflag, pointer_chain};
