use libmprompt_sys::mprompt;
use std::ffi::c_void;
use std::ptr::NonNull;
use crate::{PromptHandle, ResumeHandle};
use crate::message::Response::Yield;
use crate::prompt::{Prompt, PromptInstance};
use crate::message::TaskResult;


pub(crate) unsafe extern "C" fn __invoke_closure<F, M: Unpin, T : Prompt<M> + ?Sized>(resume: *mut mprompt::mp_resume_t, arg: *mut c_void) -> *mut c_void
    where F: FnOnce() -> T::Reply {
    let (closure, instance) = &mut *(arg as *mut (Option<F>, *mut PromptInstance<M, T>));
    match closure.take().map(|x| x()) {
        None => std::hint::unreachable_unchecked(),
        Some(result) => {
            let stamp = TaskResult(Some(Yield(
                ResumeHandle { inner: NonNull::new_unchecked(resume) },
                result,
            )));
            (**instance).storage().replace(stamp);
        }
    };
    std::ptr::null_mut()
}


pub(crate) unsafe extern "C" fn __return_one<M: Unpin, T : Prompt<M> + ?Sized>(resume: *mut mprompt::mp_resume_t, arg: *mut c_void) -> *mut c_void {
    let (reply, instance) = &mut *(arg as *mut (Option<T::Reply>, *mut PromptInstance<M, T>));
    match reply.take() {
        None => std::hint::unreachable_unchecked(),
        Some(result) => {
            let stamp = TaskResult(Some(Yield(
                ResumeHandle { inner: NonNull::new_unchecked(resume) },
                result,
            )));
            (**instance).storage().replace(stamp);
        }
    };
    std::ptr::null_mut()
}

pub(crate) unsafe extern "C" fn __run_task<M: Unpin, T : Prompt<M> + ?Sized>(parent: * mut mprompt::mp_prompt_t, arg: *mut c_void) -> *mut c_void {
    let step = &mut *(arg as *mut crate::message::Startup<M, T>);
    let task = &mut *step.task;
    let instance = &mut *step.instance;
    let result = task.run_task(PromptHandle { inner: NonNull::new_unchecked(parent), instance: step.instance }, match step.arg.take() {
        None => std::hint::unreachable_unchecked(),
        Some(x) => x
    });
    instance.storage().replace(result);
    std::ptr::null_mut()
}