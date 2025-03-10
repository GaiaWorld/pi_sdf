# pi_sdf - 基于有符号距离场(SDF)的高性能矢量图形渲染库

[![Crates.io](https://img.shields.io/crates/v/pi_sdf)](https://crates.io/crates/pi_sdf)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 功能特性

### 核心算法
- **几何处理引擎**：提供精确的圆弧、贝塞尔曲线和线段几何运算
- **轮廓生成**：支持从字体字形和SVG路径生成矢量轮廓
- **SDF生成**：基于改进的Glyphy算法实现高质量有符号距离场生成
- **多级缓存**：智能缓存系统优化高频重复计算

### 字体处理
- TTF/OTF字体解析
- 字形轮廓提取与优化
- 自动生成字形度量信息
- 支持复杂字形组合

### 图形处理
- SVG路径解析与渲染
- 矢量图形模糊效果
- 外发光特效
- 多分辨率适配

### 跨平台支持
- WebAssembly全功能支持
- 32MB内存安全分配器
- 浏览器直接渲染支持
- 高性能WebGL着色器

## 快速开始

### 安装
```toml
[dependencies]
pi_sdf = "0.1"
```

### 基本使用
```rust
use pi_sdf::{compute_near_arcs, compute_sdf_tex};

// 加载字体并生成轮廓
let buf = std::fs::read("../**.ttf");
let outline = FontFace::new(buf).to_outline('A');

// 计算近似圆弧分段
let cell_info = compute_near_arcs(&outline, 1.0);

// 生成SDF纹理
let sdf_tex = compute_sdf_tex(
    &outline,
    &cell_info,
    1024,  // 纹理尺寸
    8      // 像素范围
);
```

## WebAssembly集成

### 构建命令
```bash
cargo build --target wasm32-unknown-unknown --release
```


## SVG路径渲染
```rust
    // 构建svg路径 
    let mut path = Path::new1(
        vec![
            PathVerb::MoveTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
        ],
        vec![
            0.0, 25.33333, 0.0, 6.66666, 6.6666666, 6.666666, 0.0, 1.0, 6.666666, 0.0, 25.3333,
            0.0, 6.6666666, 6.666666, 0.0, 1.0, 32.0, 6.66666, 32.0, 25.3333, 6.666666, 6.666666,
            0.0, 1.0, 25.333333, 32.0, 6.6666, 32.0, 6.66666, 6.6666, 0.0, 1.0, 0.0, 25.3333,
        ],
    );
    // 获取svg信息
    let info = path.get_svg_info();
    // 计算svg的sdf数据
    let sdf = SvgInfo::compute_sdf_tex(&info,sdf_tex_size as usize, pxrange as u32, false, cur_off as u32, 1.0);
```
