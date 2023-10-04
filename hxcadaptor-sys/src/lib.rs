#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


#[cfg(test)]
mod test {
    use std::ffi::CString;

    use crate::hxc_strupper;

	#[test]
	fn test_string() {
		unsafe {
			let lower = CString::new("lower").unwrap();
			let upper = CString::new("LOWER").unwrap();

			let res = hxc_strupper(lower.into_raw());
			let res = CString::from_raw(res);
			assert_eq!(
				upper,
				res
			);
		}
	}
}