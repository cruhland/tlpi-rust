- Clean up util.rs syscall functions
  - Maybe provide stronger types for oflag and mode arguments?
- See if we can use move semantics to avoid reusing the buffer
- Should there be a  Drop impl for FileDescriptor at all?
  - close() can occasionally fail, so it should generally be called
    explicitly
