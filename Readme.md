This is a minimal rust wrapper over the libxcfe library.
I have implemented only the functionalities I need for my other project <https://github.com/cpcsdk/rust.cpclib>.
I have not made sound choices regarding mutability: all non mutable objects on the rustside are still mutable on the c side.

Feel free to provide patches to improve the cover of the wrapper, fix mistakes or anything else.
I can gladly provide the ownership of the repository to someone more motivated than me to continue this task.