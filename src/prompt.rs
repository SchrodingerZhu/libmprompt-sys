use crate::{PromptHandle, ResumeHandle};
use crate::message::{TaskResult, Startup, Response};
use libmprompt_sys::mprompt::{mp_prompt_enter, mp_resume, mp_prompt_create};
use crate::internal::__run_task;
use crate::prompt::PromptInstance::InProgress;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::pin::Pin;

pub trait Prompt<Message : Unpin> {
    type Reply : Unpin;

    fn run_task(&mut self, handle: PromptHandle<Message, Self>, msg: Message) -> TaskResult<Self::Reply>;

    fn create(&mut self) -> Pin<Box<PromptInstance<Message, Self>>> {
        unsafe {
            let pmt = mp_prompt_create();
            let mut instance = Box::new( PromptInstance::NotStarted(
                PromptHandle { inner: NonNull::new_unchecked(pmt), instance: std::ptr::null_mut() },
                self as *mut _,
                PhantomData::default(),
                None
            ));
            let address = instance.as_mut() as *mut _;
            match instance.as_mut() {
                PromptInstance::NotStarted(x, _, _, _) => {
                    x.instance = address
                }
                _ => std::hint::unreachable_unchecked()
            }

            Pin::new(instance)
        }
    }
}



pub enum PromptInstance<M: Unpin, T: Prompt<M> + ?Sized> {
    NotStarted(PromptHandle<M, T>, *mut T, PhantomData<M>, Option<TaskResult<T::Reply>>),
    InProgress(PromptHandle<M, T>, Option<ResumeHandle>, *mut T, Option<TaskResult<T::Reply>>),
}

impl<M: Unpin, T: Prompt<M> + ?Sized> PromptInstance<M, T> {
    pub fn next(&mut self, msg: M) -> Option<T::Reply> {
        let mut result = unsafe {
            match self {
                PromptInstance::NotStarted(handle, target, _, _) => {
                    let frame = handle.inner.as_ptr();
                    let mut startup = Startup { task: *target, arg: Some(msg), instance: self as *mut _};
                    mp_prompt_enter(frame, Some(__run_task::<M, T>), &mut startup as *mut _ as _);
                }
                PromptInstance::InProgress(_, Some(resume), _, _) => {
                    let mut resume_msg = Some(msg);
                    mp_resume(resume.inner.as_ptr(), &mut resume_msg as *mut _ as _);
                }
                _ => {
                    return None;
                }
            };
            match self.storage().take() {
                Some(x) => x,
                None => std::hint::unreachable_unchecked()
            }
        };
        Some(
            match result.get() {
                Response::Yield(handle, x) => {
                    *self = InProgress(self.prompt_handle(), Some(handle), self.underlying(), None);
                    x
                }
                Response::Finished(x) => {
                    *self = InProgress(self.prompt_handle(), None, self.underlying(), None);
                    x
                }
            }
        )
    }

    pub(crate) fn storage(&mut self) -> &mut Option<TaskResult<T::Reply>> {
        match self {
                PromptInstance::NotStarted(_, _, _, x) => { x }
                InProgress(_, _, _, x) => { x }
            }
    }

    pub fn prompt_handle(&self) -> PromptHandle<M, T> {
        PromptHandle {
            inner: match self {
                PromptInstance::NotStarted(x, _, _, _) => { x.inner }
                InProgress(x, _, _, _) => { x.inner }
            },
            instance: self as *const _ as *mut _
        }
    }

    pub fn underlying(&self) -> *mut T {
        match self {
            PromptInstance::NotStarted(_, x, _, _) => { *x }
            InProgress(_, _, x, _) => { *x }
        }
    }

    pub fn started(&self) -> bool {
        match self {
            PromptInstance::NotStarted(_, _, _, _) => false,
            PromptInstance::InProgress(_, _, _, _) => true
        }
    }
    pub fn finished(&self) -> bool {
        match self {
            PromptInstance::NotStarted(_, _, _, _) => false,
            PromptInstance::InProgress(_, x, _, _) => x.is_none()
        }
    }
}

