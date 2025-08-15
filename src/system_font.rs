

#[cfg(target_os = "android")]
pub struct FontLoader {
    parser: String,
    ext: Vec<&'static str>,
}

#[cfg(target_os = "android")]
impl FontLoader {
    pub fn new() -> Self {
        Self {
            parser: "font/SYSTEM.TTF".to_string(),
            ext: vec![
                // 通用回退字体（覆盖最广）
                "/system/fonts/NotoSansCJK-Regular.ttc",     // Android 5.0+ (CJK统一)
                "/system/fonts/NotoSerifCJK-Regular.ttc",
                "/system/fonts/DroidSansFallback.ttf", // Android 4.0-4.4
                // 厂商特定字体
                "/system/fonts/SamsungOne.ttf",          // 三星设备
                "/system/fonts/HarmonyOS-Sans.ttf",      // 华为设备
                "/system/fonts/MiSans-Regular.ttf",      // 小米设备
                "/system/fonts/OPPOSans-Regular.ttf",    // OPPO设备
                "/system/fonts/OnePlusSans-Regular.ttf", // 一加设备
                // 语言特定字体
                "/system/fonts/Roboto-Regular.ttf", // 默认拉丁字体
                "/system/fonts/NotoSerif-Regular.ttf", // 衬线字体
                // 最后尝试的备选
                "/system/fonts/NotoColorEmoji.ttf", // 表情符号
            ],
        }
    }

    /// 通过family_name查找字体系列，并返回该系列中所有字体文件的的路径。
    pub fn select_family_by_name(self, _family_name: &str) -> Result<Vec<u8>, String> {
        use std::ffi::CString;
        for path in self.ext {
            println!("read system font{}", path);
            if let Ok(data) = std::fs::read(path) {
                println!("read system font {} succeed!!", path);
                return Ok(data);
            }
        }
        log::warn!("read font ext path failed!!");
        let manager = ndk_glue::native_activity().asset_manager();
        let path = self.parser;
        let path = CString::new(path.as_str()).unwrap();
        match manager.open(path.as_c_str()) {
            Some(mut asset) => {
                if let Ok(buffer) = asset.get_buffer() {
                    return Ok(buffer.to_vec());
                } else {
                    return Err(format!("read font {:?} failed!! reason NotFound!!", path));
                }
            }
            None => {
                return Err(format!("read font {:?} failed!! reason NotFound!!", path));
            }
        }
    }
}

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Handle {
    pub path: PathBuf, // 字体文件路径
    pub font_index: u32,
}

#[cfg(target_os = "windows")]
use dwrote::Font as DWriteFont;
#[cfg(target_os = "windows")]
use dwrote::FontCollection as DWriteFontCollection;
#[cfg(not(target_arch = "wasm32"))]
use ttf_parser::fonts_in_collection;

use crate::utils::CHARS;
use crate::utils::GlyphVisitor;
use crate::utils::OutlineInfo;

#[cfg(target_os = "windows")]
pub struct FontLoader {
    system_font_collection: DWriteFontCollection,
}

#[cfg(target_os = "windows")]
impl FontLoader {
    pub fn new() -> Self {
        Self {
            system_font_collection: DWriteFontCollection::system(),
        }
    }

    /// 返回系统上安装的所有字体系列的family_name。
    pub fn all_families(&self) -> Result<Vec<String>, String> {
        Ok(self
            .system_font_collection
            .families_iter()
            .map(|dwrite_family| dwrite_family.name())
            .collect())
    }

    /// 通过family_name查找字体系列，并返回该系列中所有字体的句柄。
    pub fn select_family_by_name(&self, family_name: &str) -> Result<Vec<u8>, String> {
        let mut family = vec![];
        let dwrite_family = match self
            .system_font_collection
            .get_font_family_by_name(family_name)
        {
            Some(dwrite_family) => dwrite_family,
            None => {
                return Err(format!(
                    "read font family_name {} failed!! reason NotFound!!",
                    family_name
                ))
            }
        };
        for font_index in 0..dwrite_family.get_font_count() {
            let dwrite_font = dwrite_family.get_font(font_index);
            family.push(self.create_handle_from_dwrite_font(dwrite_font))
        }

        let path = family[1].path.to_str().unwrap().to_string();
        println!("======= reda system font: {}", path);
        match std::fs::read(&path) {
            Ok(data) => return Ok(data),
            Err(e) => return Err(format!("read font {} failed!! reason {:?}", path, e)),
        }
    }

    /// 根据DWriteFont创建Handle
    fn create_handle_from_dwrite_font(&self, dwrite_font: DWriteFont) -> Handle {
        let dwrite_font_face = dwrite_font.create_font_face();
        let dwrite_font_files = dwrite_font_face.get_files();
        Handle {
            path: dwrite_font_files[0].get_font_file_path().unwrap(),
            font_index: dwrite_font_face.get_index(),
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
pub struct SystemFont {
    pub face: ttf_parser::Face<'static>,
    _data: Vec<u8>,
    pub is_cw: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl SystemFont {
    pub fn new(data: Vec<u8>) -> Option<Self> {
        let size = fonts_in_collection(&data);
        let mut local_face = None;
        if let Some(size) = size {
            for i in 0..size {
                match ttf_parser::Face::parse(&data, i) {
                    Ok(face) => {
                        // face.glyph_phantom_points(glyph_id)
                        let font_family = face.names().into_iter().find(|n| n.name_id == 1);
                        println!("Font family: {:?}", font_family);
                        if let Some(font_family) = font_family {
                            let font_family = font_family.to_string();
                            let font_family = font_family.as_deref().unwrap_or("");
                            if font_family.contains("SC") { // 中文简体
                                local_face = Some(face);
                                break;
                            }
                        }

                    }
                    Err(err) => log::error!(
                        "ttf_parser parse font index:{} failed!! reason: {:?}",
                        i,
                        err
                    ),
                }
            }
        }
        if local_face.is_none() {
            match ttf_parser::Face::parse(&data, 0) {
                Ok(face) => local_face = Some(face),
                Err(err) => log::error!("ttf_parser parse font index:0 failed!! reason: {:?}", err),
            }
        }

        if let Some(face) = local_face {
            let mut is_cw = true;
            for c in CHARS.chars(){
                if let Some(i) = face.glyph_index(c){
                    let mut sink = GlyphVisitor::new(1.0);
                    face.outline_glyph(i, &mut sink);
                    let area = sink.get_contour_direction();
                    println!("char {} area is :{}!!!", c, area);
                    println!("==========: {}", face.has_non_default_variation_coordinates()); 
                    if area > 0.0 {
                        is_cw = false;
                    }
                    break;
                }
            }
            
            println!("========== init system font succeed!! is_cw: {}, units_per_em: {}", is_cw, face.units_per_em());
            return Some(Self {
                face: unsafe { std::mem::transmute(face) },
                _data: data,
                is_cw
            });
        }
        None
    }
}
