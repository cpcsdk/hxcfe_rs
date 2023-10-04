#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    use crate::{hxcfe_init, hxcfe_deinit};

	#[test]
	fn nothing() {unsafe{
		let res = hxcfe_init();
		hxcfe_deinit(res);
	}}
}