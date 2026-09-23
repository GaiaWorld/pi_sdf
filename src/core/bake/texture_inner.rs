/**
 * 纹理编码（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/bake/texture.rs
 */

use crate::model::base::consts::{GLYPHY_MAX_D, MAX_X, MAX_Y};
use crate::model::base::error::{Error, Result};
use crate::model::geom::curve::ArcEndpoint;
use crate::model::grid::grid::CellGrid;
use crate::model::raster::texture::{DataTexture, IndexTexture};
use crate::core::bake::arena::ArcArena;

/// 每个弧端点在数据纹理中占用的字节数（对应 pi_sdf 的一个 RGBA texel）。
const BYTES_PER_ENDPOINT: usize = 4;

/// 记录终止符：全零 texel，用于界定变长端点序列。
const TERMINATOR: [u8; BYTES_PER_ENDPOINT] = [0, 0, 0, 0];

/// 索引纹理中每个格元条目的字节数（u16 小端）。
const BYTES_PER_CELL_INDEX: usize = 2;

/// 索引条目中偏移字段的位宽。
const OFFSET_BITS: u32 = 14;

/// 索引条目中端点数字段的位宽。
const POINTS_BITS: u32 = 2;

/// 可直接编码的端点数上限；超过则以 0 表示「按终止符读到结束」。
const MAX_ENCODED_POINTS: usize = 3;

/// 数据纹理中一条记录的元信息。
struct RecordMeta {
    /// 记录在像素数组中的起始字节偏移。
    start: usize,
    /// 记录内的弧端点数（不含终止符）。
    count: usize,
}

/**
 * 把 arena 中的单位弧编码为数据纹理。
 *
 * 契约：API-033
 *
 * 约束：
 *   - requires  arena 中的端点坐标落在量化范围内
 *   - ensures   每条记录可解码为弧端点；不足 3 端点的记录补终止符；量化误差不超过半个量化步长（TERM-013）
 *   - 错误      Encode —— 量化溢出
 *
 * 参数：arena — 单位弧来源
 */
pub fn encode_data_texture(arena: &ArcArena) -> Result<DataTexture> {
    // ①逐单位弧按池序量化端点；同内容去重由 arena 的 intern（同键同下标）承担，故记录与下标一一对应
    let mut pixels = Vec::with_capacity(arena.len() * BYTES_PER_ENDPOINT);
    for index in 0..arena.len() {
        if let Some(unit) = arena.get(index) {
            for endpoint in &unit.endpoints {
                pixels.extend_from_slice(&encode_endpoint(endpoint)?);
            }
            // ②补终止符：既满足「不足 3 端点补终止符」，又为任意长度的记录提供统一定界
            pixels.extend_from_slice(&TERMINATOR);
        }
    }
    // ③每条记录占一行（宽 = 4 字节），像素总数 = 4 × 记录数
    let height = (pixels.len() / BYTES_PER_ENDPOINT) as u32;
    Ok(DataTexture {
        pixels,
        width: BYTES_PER_ENDPOINT as u32,
        height,
    })
}

/**
 * 把近邻弧网格编码为索引纹理。
 *
 * 契约：API-034
 *
 * 约束：
 *   - requires  grid 的每个单位弧已写入 data
 *   - ensures   索引指向有效数据纹理位置；填充循环保证终止
 *   - 错误      Encode —— 偏移超出位宽
 *
 * 参数：grid — 近邻弧网格
 *
 * 参数：data — 已生成的数据纹理
 */
pub fn encode_index_texture(grid: &CellGrid, data: &DataTexture) -> Result<IndexTexture> {
    // ①解码数据纹理，取得每条记录的起始偏移与端点数
    let records = decode_records(data);

    // ②格元按序对应记录；记录不足时无法给出有效索引
    if grid.cells.len() > records.len() {
        return Err(Error::Encode("数据纹理记录数少于格元数，无法建立有效索引"));
    }

    let offset_limit = 1usize << OFFSET_BITS;
    let mut pixels = Vec::with_capacity(grid.cells.len() * BYTES_PER_CELL_INDEX);

    // ③逐格元打包「端点数 + 偏移」；下标单调递增，循环有固定上界
    for cell_index in 0..grid.cells.len() {
        let record = &records[cell_index];
        let offset = record.start;

        if offset >= data.pixels.len() {
            return Err(Error::Encode("索引偏移超出数据纹理范围"));
        }
        if offset >= offset_limit {
            return Err(Error::Encode("索引偏移超出位宽"));
        }

        let num_points = if record.count > MAX_ENCODED_POINTS {
            0u16
        } else {
            record.count as u16
        };
        let entry = (num_points << (16 - POINTS_BITS)) | offset as u16;
        pixels.push((entry & 0xFF) as u8);
        pixels.push((entry >> 8) as u8);
    }

    // ④返回纹理：宽 = 2 字节，高 = 格元数
    Ok(IndexTexture {
        pixels,
        width: BYTES_PER_CELL_INDEX as u32,
        height: grid.cells.len() as u32,
    })
}

/// 量化单个坐标分量；溢出返回 Err（不 panic）。
fn quantize_coord(value: f32, max: f32) -> Result<u32> {
    if !value.is_finite() {
        return Err(Error::Encode("弧端点坐标必须为有限值"));
    }
    let quantized = value.round();
    if quantized < 0.0 || quantized > max {
        return Err(Error::Encode("弧端点坐标超出量化范围"));
    }
    Ok(quantized as u32)
}

/// 量化曲率参数：无穷远编码为 0，超出 ±GLYPHY_MAX_D 返回 Err（不 panic）。
fn encode_d(d: f32) -> Result<u8> {
    if d.is_infinite() {
        return Ok(0);
    }
    if !d.is_finite() || d.abs() > GLYPHY_MAX_D {
        return Err(Error::Encode("弧曲率参数超出编码范围"));
    }
    let id = 128.0 + (d * 127.0 / GLYPHY_MAX_D).round();
    Ok(id as u8)
}

/// 把一个弧端点编码为 4 字节 texel：[id, qx 低 8 位, qy 低 8 位, qx 高 4 位 | qy 高 4 位]。
fn encode_endpoint(endpoint: &ArcEndpoint) -> Result<[u8; BYTES_PER_ENDPOINT]> {
    let qx = quantize_coord(endpoint.px, MAX_X)?;
    let qy = quantize_coord(endpoint.py, MAX_Y)?;
    let id = encode_d(endpoint.d)?;
    let mut bytes = [0u8; BYTES_PER_ENDPOINT];
    bytes[0] = id;
    bytes[1] = (qx & 0xFF) as u8;
    bytes[2] = (qy & 0xFF) as u8;
    bytes[3] = ((((qx >> 8) & 0x0F) as u8) << 4) | ((qy >> 8) & 0x0F) as u8;
    Ok(bytes)
}

/// 按终止符切分数据纹理，返回各记录的起始偏移与端点数。
fn decode_records(data: &DataTexture) -> Vec<RecordMeta> {
    let mut records = Vec::new();
    let pixels = &data.pixels;
    let mut cursor = 0usize;

    // 游标每轮严格前进 4 字节，循环必然终止（REQ-005.1：不得出现不递减下标导致的死循环）
    while cursor + BYTES_PER_ENDPOINT <= pixels.len() {
        let start = cursor;
        let mut count = 0usize;
        while cursor + BYTES_PER_ENDPOINT <= pixels.len() {
            let chunk = &pixels[cursor..cursor + BYTES_PER_ENDPOINT];
            cursor += BYTES_PER_ENDPOINT;
            if chunk.iter().all(|byte| *byte == 0) {
                break;
            }
            count += 1;
        }
        records.push(RecordMeta { start, count });
    }
    records
}

#[cfg(test)]
mod tests {
    use super::decode_records;
    use crate::core::bake::arena::ArcArena;
    use crate::core::bake::texture::{encode_data_texture, encode_index_texture};
    use crate::model::base::consts::{GLYPHY_MAX_D, MAX_X, MAX_Y};
    use crate::model::base::error::Error;
    use crate::model::geom::aabb::Aabb;
    use crate::model::geom::curve::ArcEndpoint;
    use crate::model::geom::primitive::Point;
    use crate::model::grid::grid::{Cell, CellGrid};
    use crate::model::raster::texture::{DataTexture, UnitArc};

    /// 曲率参数量化步长。
    const D_STEP: f32 = GLYPHY_MAX_D / 127.0;

    fn endpoint(px: f32, py: f32, d: f32) -> ArcEndpoint {
        ArcEndpoint {
            px,
            py,
            d,
            tag: None,
        }
    }

    fn unit(px: f32, py: f32, d: f32) -> UnitArc {
        UnitArc {
            endpoints: vec![endpoint(px, py, d)],
            sdf_min: -1.0,
            sdf_max: 1.0,
            show: true,
        }
    }

    /// 从数据纹理解码出全部记录（终止符切分）。
    fn decode_all(data: &DataTexture) -> Vec<Vec<ArcEndpoint>> {
        let mut records = Vec::new();
        let mut current = Vec::new();
        let mut cursor = 0usize;
        while cursor + 4 <= data.pixels.len() {
            let chunk = &data.pixels[cursor..cursor + 4];
            cursor += 4;
            if chunk.iter().all(|byte| *byte == 0) {
                records.push(std::mem::take(&mut current));
                continue;
            }
            let id = chunk[0];
            let qx = (((chunk[3] >> 4) & 0x0F) as u32) << 8 | chunk[1] as u32;
            let qy = ((chunk[3] & 0x0F) as u32) << 8 | chunk[2] as u32;
            let d = if id == 0 {
                f32::INFINITY
            } else {
                (id as f32 - 128.0) * D_STEP
            };
            current.push(ArcEndpoint {
                px: qx as f32,
                py: qy as f32,
                d,
                tag: None,
            });
        }
        records
    }

    /// 构造含 count 个空格元的网格（每个格元对应一条数据记录）。
    fn make_grid(count: usize) -> CellGrid {
        let mut cells = Vec::new();
        for index in 0..count {
            let x = index as f32;
            cells.push(Cell {
                bounds: Aabb::new(Point::new(x, 0.0), Point::new(x + 1.0, 1.0)),
                arc_indices: Vec::new(),
            });
        }
        CellGrid {
            extents: Aabb::new(Point::new(0.0, 0.0), Point::new(count as f32, 1.0)),
            arcs: Vec::new(),
            cells,
            min_width: 1.0,
            min_height: 1.0,
            is_area: false,
        }
    }

    /// 构造含 count 条记录（每条 1 个端点）的数据纹理。
    fn make_data(count: usize) -> DataTexture {
        let mut arena = ArcArena::new();
        for index in 0..count {
            arena.intern(index as u64, unit((index % 4096) as f32, 0.0, 0.0));
        }
        encode_data_texture(&arena).expect("范围内的坐标应可编码")
    }

    // CASE-037：encode → 解码，端点与输入在量化容差内一致。
    #[test]
    fn case_037_data_texture_roundtrip_within_half_step() {
        let mut arena = ArcArena::new();
        arena.intern(
            1,
            UnitArc {
                endpoints: vec![
                    endpoint(0.0, 0.0, 0.0),
                    endpoint(100.0, 200.0, 0.25),
                    endpoint(MAX_X, MAX_Y, -0.25),
                ],
                sdf_min: -1.0,
                sdf_max: 1.0,
                show: true,
            },
        );
        arena.intern(
            2,
            UnitArc {
                endpoints: vec![endpoint(12.0, 34.0, GLYPHY_MAX_D)],
                sdf_min: -1.0,
                sdf_max: 1.0,
                show: true,
            },
        );

        let data = encode_data_texture(&arena).expect("范围内坐标应可编码");
        // 尺寸不变量：像素长度 = 宽 × 高。
        assert_eq!(data.pixels.len() as u32, data.width * data.height);
        // 6 个 texel（3+1 端点 + 两条终止符）× 4 字节。
        assert_eq!(data.pixels.len(), 24);

        let decoded = decode_all(&data);
        assert_eq!(decoded.len(), 2, "两条记录应各留下一条终止标记");
        assert_eq!(decoded[0].len(), 3);
        assert_eq!(decoded[1].len(), 1);

        let expected = [
            (0.0f32, 0.0f32, 0.0f32),
            (100.0, 200.0, 0.25),
            (MAX_X, MAX_Y, -0.25),
        ];
        for (orig, got) in expected.iter().zip(decoded[0].iter()) {
            assert!(
                (orig.0 - got.px).abs() <= 0.5,
                "px 量化误差应不超过半步长: {} vs {}",
                orig.0,
                got.px
            );
            assert!((orig.1 - got.py).abs() <= 0.5, "py 量化误差应不超过半步长");
            assert!(
                (orig.2 - got.d).abs() <= D_STEP * 0.5 + 1e-6,
                "d 量化误差应不超过半步长: {} vs {}",
                orig.2,
                got.d
            );
        }
        assert!((decoded[1][0].d - GLYPHY_MAX_D).abs() <= D_STEP * 0.5 + 1e-6);
    }

    // CASE-038：坐标超出量化范围时返回 Err，不 panic。
    #[test]
    fn case_038_quantize_overflow_returns_err() {
        let mut overflow = ArcArena::new();
        overflow.intern(1, unit(MAX_X + 1.0, 0.0, 0.0));
        assert!(matches!(
            encode_data_texture(&overflow),
            Err(Error::Encode(_))
        ));

        let mut negative = ArcArena::new();
        negative.intern(1, unit(-1.0, 0.0, 0.0));
        assert!(matches!(
            encode_data_texture(&negative),
            Err(Error::Encode(_))
        ));

        let mut too_tall = ArcArena::new();
        too_tall.intern(1, unit(0.0, MAX_Y + 1.0, 0.0));
        assert!(matches!(
            encode_data_texture(&too_tall),
            Err(Error::Encode(_))
        ));

        let mut curved = ArcArena::new();
        curved.intern(1, unit(0.0, 0.0, GLYPHY_MAX_D + 0.1));
        assert!(matches!(encode_data_texture(&curved), Err(Error::Encode(_))));

        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut arena = ArcArena::new();
            arena.intern(1, unit(bad, 0.0, 0.0));
            assert!(
                matches!(encode_data_texture(&arena), Err(Error::Encode(_))),
                "非有限坐标 {} 应返回 Err",
                bad
            );
        }

        // 边界值恰在范围内时应成功。
        let mut edge = ArcArena::new();
        edge.intern(1, unit(MAX_X, MAX_Y, GLYPHY_MAX_D));
        assert!(encode_data_texture(&edge).is_ok(), "边界坐标应可编码");
    }

    // CASE-039：有限时间内返回，无死循环。
    #[test]
    fn case_039_index_texture_terminates() {
        let grid = make_grid(5);
        let data = make_data(5);
        let index = encode_index_texture(&grid, &data).expect("记录充足时应有限返回");
        assert_eq!(index.height, 5);
        assert_eq!(index.pixels.len() as u32, index.width * index.height);

        // 记录不足也必须迅速返回 Err，而不是进入不递减下标的死循环。
        let crowded = make_grid(6);
        assert!(matches!(
            encode_index_texture(&crowded, &data),
            Err(Error::Encode(_))
        ));

        // 零格元：返回自洽空纹理。
        let empty_grid = make_grid(0);
        let empty_index = encode_index_texture(&empty_grid, &data).expect("零格元应返回 Ok");
        assert!(empty_index.pixels.is_empty());
        assert_eq!(empty_index.height, 0);
    }

    // CASE-040：逐项解引用，每个索引指向有效数据位置。
    #[test]
    fn case_040_index_points_to_valid_data_position() {
        let grid = make_grid(4);
        let data = make_data(4);
        let index = encode_index_texture(&grid, &data).expect("应成功");
        let records = decode_records(&data);
        assert_eq!(records.len(), 4, "应正好四条记录");

        for cell in 0..grid.cells.len() {
            let entry =
                u16::from_le_bytes([index.pixels[2 * cell], index.pixels[2 * cell + 1]]);
            let offset = (entry & ((1u16 << 14) - 1)) as usize;
            let num_points = entry >> 14;

            assert!(offset < data.pixels.len(), "索引必须指向有效数据位置");
            assert_eq!(offset % 4, 0, "偏移应对齐到 texel");
            assert_eq!(offset, records[cell].start, "偏移应指向对应记录起点");
            assert!(
                data.pixels[offset..offset + 4].iter().any(|byte| *byte != 0),
                "记录起点不应是终止符"
            );

            let expected_points = if records[cell].count > 3 {
                0u16
            } else {
                records[cell].count as u16
            };
            assert_eq!(num_points, expected_points);
        }
    }

    // CASE-080：偏移超出位宽返回 Err(Encode)，不 panic。
    #[test]
    fn case_080_offset_overflow_returns_encode_error() {
        // 首条记录 4096 个端点 = 16384 字节，再加终止符，使第二条记录起点越过 14 位偏移上限。
        let mut endpoints = Vec::with_capacity(4096);
        for index in 0..4096usize {
            endpoints.push(endpoint((index % 4096) as f32, 0.0, 0.0));
        }
        let mut arena = ArcArena::new();
        arena.intern(
            0,
            UnitArc {
                endpoints,
                sdf_min: 0.0,
                sdf_max: 0.0,
                show: true,
            },
        );
        arena.intern(1, unit(1.0, 1.0, 0.0));

        let data = encode_data_texture(&arena).expect("应可编码");
        assert!(data.pixels.len() > (1usize << 14));

        let grid = make_grid(2);
        match encode_index_texture(&grid, &data) {
            Err(Error::Encode(_)) => {}
            other => panic!(
                "偏移超出位宽应返回 Err(Encode)，实际 {:?}",
                other.map(|texture| texture.pixels.len())
            ),
        }
    }
}
