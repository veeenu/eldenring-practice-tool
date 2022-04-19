#![feature(once_cell)]

pub mod memedit;
pub mod codegen;
pub mod pointers;
pub mod version;

pub mod prelude {
    pub use crate::memedit::*;
    pub use crate::pointers::*;
    pub use crate::version::*;
    pub use crate::codegen::*;
}
