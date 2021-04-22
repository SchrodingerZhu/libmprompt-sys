use crate::ResumeHandle;
use crate::prompt::{Prompt, PromptInstance};

pub(crate) struct Startup<M : Unpin, T: Prompt<M> + ?Sized> {
    pub(crate) task: *mut T,
    pub(crate) arg: Option<M>,
    pub(crate) instance: *mut PromptInstance<M, T>
}

pub struct TaskResult<T> (pub(crate) Option<Response<T>>);

pub enum Response<T> {
    Yield(ResumeHandle, T),
    Finished(T),
}

pub fn finished<T>(data: T) -> TaskResult<T> {
    return TaskResult(Some(Response::Finished(data)));
}

impl<T> TaskResult<T> {
    pub(crate) fn get(&mut self) -> Response<T> {
        unsafe {
            match self.0.take() {
                None => std::hint::unreachable_unchecked(),
                Some(x) => x
            }
        }
    }
}

