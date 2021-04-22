#![allow(non_camel_case_types, non_upper_case_globals)]
#![cfg_attr(test, allow(unknown_lints, deref_nullptr))]
#![no_std]

pub mod mprompt;
#[cfg(feature = "mpeff")]
pub mod mpeff;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::mprompt::*;
    use std::ffi::c_void;

    pub const N: usize = 1000;
    pub const M: usize = 1000000;

    unsafe extern "C" fn await_result(r: *mut mp_resume_t, _arg: *mut c_void) -> *mut c_void {
        return r as _;
    }

    unsafe extern "C" fn async_worker(parent: *mut mp_prompt_t, _arg: *mut c_void) -> *mut c_void {
        let mut partial_result : usize = 0;
        mp_yield(parent, Some(await_result), 0 as _);
        partial_result += 1;
        return partial_result as _;
    }

    #[test]
    fn async_workers() {
        unsafe {
            let mut workers = std::vec::Vec::new();
            workers.resize(N, std::ptr::null_mut());
            let mut count = 0_usize;
            for i in 0..M + N {
                let j = i % N;
                if workers[j] as usize != 0_usize {
                    count += mp_resume(workers[j], 0 as _) as usize;
                    workers[j] = 0 as _;
                }
                if i < M {
                    workers[j] = mp_prompt(Some(async_worker), 0 as _) as _;
                }
                std::println!("ran {} workers", count);
            }
        }
    }


}

