// By default on Linux, `rand` makes direct `getrandom` syscalls, which we can't patch. Using the
// "linux_getrandom" backend makes it call into libc instead.
#[cfg(all(
    target_os = "linux",
    not(getrandom_backend = "linux_getrandom"),
    not(doc),
    not(doctest),
))]
compile_error!("This crate requires `--cfg getrandom_backend=\"linux_getrandom\"");

mod context;
mod error;
mod patch;
mod sim;

#[cfg(feature = "cli")]
pub mod cli;

pub use crate::error::{Error, Result};
pub use crate::sim::Sim;
