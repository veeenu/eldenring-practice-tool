#![feature(once_cell)]

pub mod memedit;
pub mod base_addresses;
pub mod pointers;
pub mod version;

pub mod prelude {
    pub use crate::memedit::*;
    pub use crate::pointers::*;
    pub use crate::version::*;
}
