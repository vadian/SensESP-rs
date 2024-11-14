use async_channel::unbounded;
use async_executor::Executor;
use easy_parallel::Parallel;
use futures_lite::future;

pub fn get_executor() {
let ex = Executor::new();
let (signal, shutdown) = unbounded::<()>();

Parallel::new()
    // Run four executor threads.
    .each(0..4, |_| future::block_on(ex.run(shutdown.recv())))
    // Run the main future on the current thread.
    .finish(|| future::block_on(async {
        println!("Hello world!");
        drop(signal);
    }));
}

//todo:
//I think the .finish needs to be called after everything else has started, or perhaps not at all?
//look into the 'main future'.
//perhaps a timer is the hardest thing I could start with :D 
