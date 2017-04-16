#![macro_use]
//! Registers a function to be called before main (if an executable) or when loaded (if a dynamic
//! library).
//! **Using this is a bad idea.** You can do this in a way that isn't abysmally fragile.
//! 
//! Example
//! =======
//! 
//! ```
//! # #[macro_use] extern crate constructor;
//! pub static mut X: usize = 0;
//! 
//! extern fn init() {
//!     unsafe { X = 5; }
//! }
//! constructor! { init }
//! 
//! 
//! fn main() {
//!    assert_eq!(unsafe { X }, 5);
//! }
//! ```
//! 
//! Caveats
//! =======
//! This isn't exactly portable, though the implementation is quite simple.
//! 
//! Doing anything particularly complicated, such IO or loading libraries, may cause problems
//! on Windows. (?)
//! 
//! Every parent module must be `pub`lic. If it is not, then it will be
//! stripped out by `--release`. At least the compiler gives a helpful warning.
//! 
//! 
//! 
//! Beware, for some say that these techniques can unleash a terrible evil.
//! [lazy_static](https://crates.io/crates/lazy_static) may be a more appropriate tool.

#[macro_export]
macro_rules! constructor {
    ($($FN:ident),*) => {
        $(pub mod $FN {
            #![allow(non_snake_case)]
            #![allow(dead_code)]
            #![allow(non_upper_case_globals)]
            #![deny(private_no_mangle_statics /* >>> constructor must be used from a pub mod <<< */)]
            // (The comment above is to make the error message more meaningful.)

            // http://stackoverflow.com/questions/35428834/how-to-specify-which-elf-section-to-use-in-a-rust-dylib
            // https://msdn.microsoft.com/en-us/library/bb918180.aspx
            // Help given by WindowsBunny!

            #[cfg(target_os = "linux")]
            #[link_section = ".ctors"]
            #[no_mangle]
            pub static $FN: extern fn() = super::$FN;

            // FIXME: macos untested
            #[cfg(target_os = "macos")]
            #[link_section = "__DATA,__mod_init_func"]
            #[no_mangle]
            pub static $FN: extern fn() = super::$FN;

            // FIXME: windows untested; may require something more complicated for certain target
            // triples?
            #[cfg(target_os = "windows")]
            #[link_section = ".CRT$XCU"]
            #[no_mangle]
            pub static $FN: extern fn() = super::$FN;

            // We could also just ignore cfg(target_os) & have 1 item, but this way we'll have a compilation error if we don't know `target_os`.
        })*
    };
}

#[cfg(test)]
pub mod test {
    static mut BUNNY: &'static str = "i'm just a cute li'l bunny!\nand i wont hurt nobody!!\n🐰";

    #[test]
    fn bunny_is_fluffy() {
        println!("{}", unsafe { BUNNY });
    }



    pub static mut RAN: bool = false;
    extern "C" fn set_ran() {
        unsafe { RAN = true }
    }
    constructor! { set_ran }

    #[test]
    fn works() {
        assert!(unsafe { RAN });
    }



    extern crate zalgo;
    constructor! { corrupt_bunny }
    pub extern "C" fn corrupt_bunny() {
        unsafe {
            use self::zalgo::*;
            let mr_bun = gen(BUNNY, false, false, true, ZalgoSize::None);
            let red = "\x1b[31m";
            let reset = "\x1b[0m";
            let mr_bun = format!("{}{}{}", red, mr_bun, reset);

            use std::mem::{transmute, forget};
            {
                let mr_bun: &str = &mr_bun;
                BUNNY = transmute(mr_bun);
            }
            forget(mr_bun); // u din't see nuffin'
        }
        // ... okay there's probably a terminal somewhere that does a decent job of this.
    }
}
