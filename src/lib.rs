use std::{ptr, sync::RwLock};

use freetype_sys::FT_Init_FreeType;

#[macro_use]
extern crate lazy_static;

pub mod glyphy;
pub mod glyphy_draw;
pub mod utils;

lazy_static! {
    // ft_lib 全局指针，所有字体共用本字体
    pub static ref FT_LIB: RwLock<u64> = {
        let mut ft_lib = ptr::null_mut();
        unsafe {
            assert_eq!(FT_Init_FreeType(&mut ft_lib), 0);
        }
        RwLock::new(ft_lib as u64)
    };
}