
use crate::{ glyphy::geometry::aabb::Aabb, Point, Vector2};

/// 计算误差函数 (Error Function) erf(x)。
///
/// erf函数是概率论和统计学中的一个特殊函数，用于描述从负无穷到x的正态分布的积分。
/// 这个函数在信号处理、量子力学等领域也有广泛应用。
///
/// 参数：
/// - x: 输入的浮点数。
///
/// 返回值：
/// erf(x)的计算结果。
///
/// 如果x为负数，则返回结果为负；否则返回正值。
fn erf(mut x: f32) -> f32 {
    // 确定输入是否为负数，并处理x
    let negative = x < 0.0;
    if negative {
        x = -x; // 转换为正数处理
    }

    // 计算x的x2, x3, x4
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x2 * x2;

    // 计算分母denom
    let denom = 1.0 + 0.278393 * x + 0.230389 * x2 + 0.000972 * x3 + 0.078108 * x4;

    // 计算结果result
    let result = 1.0 - 1.0 / (denom.powi(4));

    // 处理结果的正负
    if negative {
        -result
    } else {
        result
    }
}


/// 计算误差函数，使用标准差sigma缩放输入。
///
/// 该函数通过将输入x除以sigma乘以sqrt(2)来缩放x，然后调用erf计算。
///
/// 参数：
/// - x: 输入浮点数
/// - sigma: 标准差，用于缩放x
///
/// 返回值：
/// erf(x / (sigma * sqrt(2)))的值
fn erf_sigma(x: f32, sigma: f32) -> f32 {
    erf(x / (sigma * 1.4142135623730951))
}

/// 根据矩形的两个对角点计算颜色值。
///
/// 该函数利用erf_sigma函数分别计算x和y的边界，并将差值进行相乘后再除以4得到最终结果。
///
/// 参数：
/// - p0: 矩形的对角点1
/// - p1: 矩形的对角点2
/// - sigma: 标准差，用于缩放输入
///
/// 返回值：
/// 计算出的颜色值，范围在[0,1]之间
fn color_from_rect(p0: Vector2, p1: Vector2, sigma: f32) -> f32 {
    (erf_sigma(p1.x, sigma) - erf_sigma(p0.x, sigma))
        * (erf_sigma(p1.y, sigma) - erf_sigma(p0.y, sigma))
        / 4.0
}

/// 计算阴影的透明度值alpha。
///
/// 根据给定位置pos，在矩形区域从pt_min到pt_max之间生成阴影的透明度。
/// 该函数通过计算pos到pt_min和pos到pt_max的距离，然后调用color_from_rect函数获得颜色值，
/// 其中alpha由这个颜色值决定。
///
/// 参数：
/// - pos: 当前点位置
/// - pt_min: 矩形区域的最小坐标点
/// - pt_max: 矩形区域的最大坐标点
/// - sigma: 标准差，用于缩放输入
///
/// 返回值：
/// 阴影的alpha值，范围在[0,1]之间
fn get_shadow_alpha(pos: Point, pt_min: &Point, pt_max: &Point, sigma: f32) -> f32 {
    let d_min = pos - pt_min;  // 计算pos到pt_min的向量
    let d_max = pos - pt_max;  // 计算pos到pt_max的向量

    return color_from_rect(d_min, d_max, sigma);  // 调用color_from_rect得到alpha值
}

/// 包含模糊处理相关信息的结构体。
///
/// 该结构体保存了模糊处理所需的纹理数据、宽度、高度和包围盒信息。
///
/// 成员：
/// - tex: 存放纹理数据的字节容器
/// - width: 纹理的宽度（以像素为单位）
/// - height: 纹理的高度（以像素为单位）
/// - bbox: 包围盒，用于空间索引或其他计算
pub struct BlurInfo {
    pub tex: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub bbox: Vec<f32>,
}

/// 生成模糊框的函数。
///
/// 该函数根据输入的包围盒bbox、像素范围pxrange和纹理盒子尺寸txe_size，
/// 生成一个BlurInfo结构体，其中包括模糊处理所需的纹理数据、宽度、高度和新的包围盒。
///
/// 参数：
/// - bbox: 输入的包围盒，由4个浮点数组成，分别是左下、右上坐标。
/// - pxrange: 像素范围，决定生成的模糊区域大小。
/// - txe_size: 纹理盒子的大小，单位为像素。
///
/// 返回值：
/// 一个BlurInfo结构体，包含模糊处理后的纹理数据、宽度、高度和新的包围盒。
pub fn blur_box(bbox: &[f32], pxrange: f32, txe_size: usize) -> BlurInfo {
    // 创建包围盒实例
    let bbox = Aabb::new(Point::new(bbox[0], bbox[1]), Point::new(bbox[2], bbox[3]));
    let b_w = bbox.width();
    let b_h = bbox.height();

    // 计算像素间距
    let px_dsitance = b_h.max(b_w) / (txe_size - 1) as f32; // 两边加上pxrange + 0.5，中间减一

    // 计算需要生成的像素数
    let px_num = pxrange.ceil();
    let px_num2 = px_num + 0.5;
    let sigma = px_num / 6.0;
    let dsitance = px_dsitance * (px_num);

    // 计算生成的纹理尺寸
    let p_w = (b_w / px_dsitance).ceil() + px_num2 * 2.0;
    let p_h = (b_h / px_dsitance).ceil() + px_num2 * 2.0;

    // 初始化像素图
    let mut pixmap = vec![0; (p_w * p_h) as usize];

    // 计算起始点
    let start = Point::new(bbox.mins.x - dsitance, bbox.mins.y - dsitance);
    // log::debug!("{:?}", start);
    let mut pos = Point::default();
    for i in 0..p_w as usize {
        for j in 0..p_h as usize {
            pos = Point::new(
                start.x + i as f32 * px_dsitance,
                start.y + j as f32 * px_dsitance,
            );
            // log::debug!("pos: {}", pos);
            let a = get_shadow_alpha(pos, &bbox.mins, &bbox.maxs, sigma);
            pixmap[j * p_w as usize + i as usize] = (a * 255.0) as u8;
        }
    }

    // 计算新的包围盒的最大点
    let maxs = if b_h > b_w {
        Point::new(b_w / px_dsitance + px_num2, p_h - px_num2)
    } else {
        Point::new(p_w - px_num2, b_h / px_dsitance + px_num2)
    };

    // 创建新的包围盒实例
    let atlas_bounds = Aabb::new(
        Point::new(px_num2, px_num2),
        maxs,
    );
    log::debug!("atlasBounds: {:?}", atlas_bounds);

    // 返回BlurInfo结构体
    BlurInfo {
        tex: pixmap,
        width: p_w as usize,
        height: p_h as usize,
        bbox: vec![px_num, px_num, p_w - px_num, p_h - px_num],
    }
}
const SCALE: f32 = 10.0;

// 分析代码并生成中文文档

/// 高斯模糊处理函数
///
/// 该函数对输入的RGBA图像进行高斯模糊处理，生成输出图像
///
/// 参数：
/// - sdf_tex：输入图像的字节数据，假设为RGBA格式，每个像素占用4个字节
/// - width：输入图像的宽度（单位：像素）
/// - height：输入图像的高度（单位：像素）
/// - radius：高斯模糊的半径，决定模糊程度
/// - weight：权重调整系数，用于控制模糊强度
///
/// 返回值：
/// 处理后的图像字节数据，输出与输入格式相同
pub fn gaussian_blur(
    sdf_tex: Vec<u8>,
    width: u32,
    height: u32,
    radius: u32,
    weight: f32,
) -> Vec<u8> {
    // let (width, height) = img.dimensions();
    let mut output = Vec::with_capacity(sdf_tex.len());
    let weight = -weight / SCALE;
    let kernel = create_gaussian_kernel(radius);
    let kernel_size = kernel.len() as u32;

    for y in 0..height {
        for x in 0..width {
            // let mut r = 0.0;
            // let mut g = 0.0;
            // let mut b = 0.0;
            let mut a = 0.0;
            let mut weight_sum = 0.0;

            for ky in 0..kernel_size {
                for kx in 0..kernel_size {
                    let px =
                        (x as i32 + kx as i32 - radius as i32).clamp(0, width as i32 - 1) as u32;
                    let py =
                        (y as i32 + ky as i32 - radius as i32).clamp(0, height as i32 - 1) as u32;

                    let sdf = sdf_tex[(px + py * width) as usize] as f32 / 255.0;
                    let fill_sd_px = sdf - (0.5 + weight);
                    let pixel = (fill_sd_px + 0.5).clamp(0.0, 1.0);

                    let weight = kernel[ky as usize][kx as usize];

                    // r += pixel[0] as f32 * weight;
                    // g += pixel[1] as f32 * weight;
                    // b += pixel[2] as f32 * weight;
                    a += pixel as f32 * weight;
                    weight_sum += weight;
                }
            }

            let pixel = (a / weight_sum * 255.0) as u8;

            output.push(pixel);
        }
    }

    output
}
/// 创建高斯模糊核函数的函数，用于生成高斯模糊矩阵
///
/// 输入参数：模糊半径radius，决定高斯核的大小。
/// 核的大小为2*radius +1，这是一个奇数大小的矩阵，中心为原点，用于对称地应用模糊效果。
/// 该函数返回一个二维的f32数组，即高斯核的权重矩阵，所有权重之和为1，可直接用于图像的卷积运算，实现高斯模糊效果。
/// 其中，较大的sigma值对应于较大程度的模糊，较小的sigma则近似于较为清晰的图像。同时，sigma的值计算为radius / 2，这确保了核函数的合适衰减，
/// 以达到平滑的目的。生成核的具体步骤包括：1、初始化核矩阵；2、计算每个点的高斯值；3、归一化，使得所有权重之和为1，方便后续的卷积运算。
/// 具体计算公式为：value = exp(- (dx*dx + dy*dy) / (2*sigma*sigma)) / (2*π*sigma*sigma)，其中dx和 dy为相对于中心的距离，计算点为(x - radius, y - radius)，其中x, y为当前核矩阵的位置索引。
/// 归一化后，权重矩阵的每个元素除以总和，使得整个核的带来的增益保持一致，从而避免图像亮度的变化。
/// 返回的核矩阵是一个二维数组，形状为(size, size)的方阵，其中size=2*radius+1，用于对二维图像进行卷积操作，实现高斯模糊效果的生成。
/// 该高斯核的生成方法通常在图像处理、计算机视觉等领域中广泛使用，是实现高斯模糊的标准方法之一。
fn create_gaussian_kernel(radius: u32) -> Vec<Vec<f32>> {
    let sigma = radius as f32 / 2.0;
    let size = radius * 2 + 1;
    let mut kernel = vec![vec![0.0; size as usize]; size as usize];
    let mut sum = 0.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - radius as f32;
            let dy = y as f32 - radius as f32;
            let value = (-((dx * dx + dy * dy) / (2.0 * sigma * sigma))).exp()
                / (2.0 * std::f32::consts::PI * sigma * sigma);
            kernel[y as usize][x as usize] = value;
            sum += value;
        }
    }

    for y in 0..size {
        for x in 0..size {
            kernel[y as usize][x as usize] /= sum;
        }
    }

    kernel
}


pub fn blur_box2(info: BoxInfo) -> Vec<u8> {
    let BoxInfo {
        p_w,
        p_h,
        start,
        px_dsitance,
        sigma,
        bbox,
        ..
    } = info;
    let mut pixmap = vec![0; (p_w * p_h) as usize];
    let start = Point::new(0.5,0.5); // 将起点设置为(0.5, 0.5)以确保像素中心对齐
    for i in 0..p_w as usize {  // 遍历每个目标像素的宽度坐标
        for j in 0..p_h as usize {  // 遍历每个目标像素的高度坐标
            let pos = Point::new(  // 计算当前目标像素中心的位置坐标
                start.x + i as f32,  // pos.x = 起始x坐标 + i（转换为f32以便计算）
                start.y + j as f32,  // pos.y = 起始y坐标 + j（同上转换为f32）
            );
            let a = get_shadow_alpha(pos, &bbox.mins, &bbox.maxs, sigma);  // 计算当前位置处的模糊alpha值
            pixmap[j * p_w as usize + i as usize] = (a * 255.0) as u8;  // 将alpha值转换为0-255整数，存储到 pixmap 中对应位置
        }
    }
    pixmap
}

#[derive(Debug, Clone)]
pub struct BoxInfo {
    pub p_w: f32,  // 输出图标的宽度（单位：像素）
    pub p_h: f32,  // 输出图标的高度（单位：像素）
    start: Point,  // 图标绘制的起始点坐标
    px_dsitance: f32,  // 每个像素之间的间距
    sigma: f32,  // 高斯模糊的标准差
    pub atlas_bounds: Aabb,  // 图标的包围盒在atlas中的位置
    bbox: Aabb,  // 图标的内容包围盒
    pub radius: u32  // 高斯模糊的半径，即输入参数radius，用于计算核矩阵的大小
}

/// 计算盒子布局信息的函数
///
/// 该函数根据输入的包围盒信息（bbox）、atlas大小（txe_size）和高斯模糊半径（radius），计算并返回BoxInfo结构体，用于后续处理。
///
/// 参数：
/// - bbox：目标物体的包围盒，包含最小（mins）和最大（maxs）坐标
/// - txe_size：atlas的大小，用于计算像素间距
/// - radius：高斯模糊处理的半径，决定模糊程度
///
/// 返回值：
/// BoxInfo结构体，包含输出图标的尺寸、起始点、像素间距、sigma值、atlas边界、内容边界框以及模糊半径等信息
pub fn compute_box_layout(bbox: Aabb, txe_size: usize, radius: u32) -> BoxInfo {
    let b_w = bbox.maxs.x - bbox.mins.x;  // 计算包围盒的宽度
    let b_h = bbox.maxs.y - bbox.mins.y;  // 计算包围盒的高度

    // 计算像素间距：取宽度和高度的最大值，除以atlas像素数减一，以适应边距
    let px_dsitance = b_h.max(b_w) / (txe_size - 1) as f32;
    // 计算像素数量（考虑模糊半径的影响）
    let px_num = radius as f32;  // 将半径转换为f32
    let px_num2 = px_num + 0.5;  // 进行0.5的偏移，以确保正确对齐
    // 计算sigma：根据高斯模糊的标准差公式，sigma通常设为半径的三分之一，以获得合适的模糊效果
    let sigma = px_num / 3.0;
    // 计算距离：像素间距乘以模糊半径，用于确定绘制区域的扩展范围
    let dsitance = px_dsitance * px_num;

    // 计算输出图标的宽度和高度
    let p_w = (b_w / px_dsitance).ceil() + px_num2 * 2.0;  // 确保能够涵盖整个包围盒，并扩展到模糊半径的两边
    let p_h = (b_h / px_dsitance).ceil() + px_num2 * 2.0;

    // 确定绘制的起始点，通常在包围盒最小坐标的左边，减去模糊距离，确保内容完整
    let start = Point::new(bbox.mins.x - dsitance, bbox.mins.y - dsitance);

    // 计算atlas中的最大点坐标，确保内容正确放置
    let maxs = if b_h > b_w {  // 根据宽度和高度的比较，决定扩展方式
        Point::new(b_w / px_dsitance + px_num2, p_h - px_num2)
    } else {
        Point::new(p_w - px_num2, b_h / px_dsitance + px_num2)
    };

    // 创建atlas的包围盒信息
    let atlas_bounds = Aabb::new(Point::new(px_num2, px_num2), maxs);
    let info: BoxInfo  =BoxInfo {
        p_w,
        p_h,
        start,
        px_dsitance,
        sigma,
        atlas_bounds,
        bbox: atlas_bounds,
        radius
    };
    log::debug!("BoxInfo: {:?}", info);
    info

}
