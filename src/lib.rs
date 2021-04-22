#![feature(fn_traits)]

use libmprompt_sys::mprompt;
use libmprompt_sys::mprompt::mp_yield;
use std::ptr::NonNull;
use crate::internal::{__invoke_closure, __return_one};
use crate::prompt::{Prompt, PromptInstance};

pub mod message;
pub mod prompt;
mod internal;

#[derive(Copy, Clone)]
pub struct ResumeHandle {
    inner: NonNull<mprompt::mp_resume_t>
}

#[derive(Copy, Clone)]
pub struct PromptHandle<M: Unpin, T : Prompt<M> + ?Sized> {
    inner: NonNull<mprompt::mp_prompt_t>,
    instance: *mut PromptInstance<M, T>
}

impl<M: Unpin, T : Prompt<M> + ?Sized> PromptHandle<M, T> {
    pub fn yield_with<F>(&self, f: F) -> M
        where F: FnOnce() -> T::Reply {
        let f = (Some(f), self.instance);
        unsafe {
            let result = mp_yield(self.inner.as_ptr(), Some(__invoke_closure::<F, M, T>), &f as *const _ as _) as *mut Option<M>;
            match (*result).take() {
                None => std::hint::unreachable_unchecked(),
                Some(x) => x
            }
        }
    }

    pub fn yield_one(&self, f: T::Reply) -> M {
        let f =  (Some(f), self.instance);
        unsafe {
            let result = mp_yield(self.inner.as_ptr(), Some(__return_one::<M, T>), &f as *const _ as _) as *mut Option<M>;
            match (*result).take() {
                None => std::hint::unreachable_unchecked(),
                Some(x) => x
            }
        }
    }
}




#[cfg(test)]
mod test {
    use crate::prompt::Prompt;
    use crate::PromptHandle;
    use crate::message::{finished, TaskResult};

    #[test]
    fn counter_test() {
        struct Counter {
            cnt: usize
        }

        impl Prompt<usize> for Counter {
            type Reply = usize;

            fn run_task(&mut self, handle: PromptHandle<Self::Reply, Self>, msg: usize) -> TaskResult<Self::Reply> {
                self.cnt += msg;
                for _ in 0..100 {
                    let msg: usize = handle.yield_with(|| self.cnt);
                    self.cnt += msg;
                }
                finished( self.cnt as _)
            }
        }

        let mut counter = Counter { cnt: 1 };
        let mut instance = counter.create();
        while let Some(x) = instance.next(1) {
            println!("{}", x);
        }
        assert_eq!(counter.cnt, 102)
    }

    #[test]
    fn multi_test() {
        struct Tester;

        impl Prompt<usize> for Tester {
            type Reply = usize;

            fn run_task(&mut self, _handle: PromptHandle<Self::Reply, Self>, msg: usize) -> TaskResult<Self::Reply> {
                let mut result = 1;
                if msg > 0 {
                    let mut inner = Tester;
                    if let Some(x) = inner.create().next(msg - 1) {
                        result += x;
                    }
                }
                finished( result)
            }
        }

        let mut tester = Tester;
        let mut instance = tester.create();
        let x = instance.next(10).unwrap();
        {
            println!("{}", x);
            assert_eq!(11, x);
        }
    }
}
