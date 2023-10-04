# hxc_rs

This is a minimal rust wrapper over the libxcfe library <https://github.com/jfdelnero/HxCFloppyEmulator>.
I have only implemented the functionalities needed for my other project <https://github.com/cpcsdk/rust.cpclib> (mainly to allow  [basm assembler](https://cpcsdk.github.io/rust.cpclib/basm/) to write in HFE image discs).
I have not made sound choices regarding mutability: all non mutable objects on the rust-side are still mutable on the c-side.
I may have memory leaks, even if I tried to avoid them.


Feel free to provide patches to improve the cover of the wrapper, fix mistakes, or anything else.
I can gladly provide the ownership of the repository to someone more motivated than me to continue this task (I will only add what I need for my main project).


`x86_64-pc-windows-gnu` is the required toolchain for windows