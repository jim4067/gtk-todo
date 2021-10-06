use std::future::Future;

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        ctx.push_thread_default();
        ctx
    }) //did he forget to add ; here
}

pub fn spawn<F: Future<Output = ()> + 'static>(future: F) {
    glib::MainContext::default().spawn_local(future);
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    thread_context().block_on(future)
}