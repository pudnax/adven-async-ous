# adven-async-ous
Custom threadpool for async tasks in Rust

1. We need a thread pool to execute our CPU intensive tasks or tasks that we want too run asynchronously but not in our OS backed event queue
2. We need to make a simple cross platform epoll/kqueue/IOCP event loop. Now this turns out to be extremely interesting, but it's also a lot of code, so I split this section off into a separate "companion book" for those that want to explore this further. We use this library here called minimio.
