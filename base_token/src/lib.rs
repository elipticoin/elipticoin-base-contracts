#![feature(alloc, proc_macro_hygiene)]

#[cfg(not(test))]
extern crate wee_alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
extern crate alloc;
#[cfg(not(test))]
extern crate ellipticoin;
extern crate wasm_rpc;
extern crate wasm_rpc_macros;
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate ellipticoin_test_framework;
#[cfg(test)]
extern crate mock_ellipticoin as ellipticoin;

pub mod base_token;
mod issuance;
mod errors;
