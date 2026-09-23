/**
 * 边界编解码（字节通道）
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::model::base::error::Result;
use crate::api::codec_inner;

/**
 * 把值编码为边界字节。
 *
 * 契约：API-032
 *
 * 约束：
 *   - requires  T 实现序列化框架的对应特征
 *   - ensures   编解码只出现在字节通道；失败返回 Err
 *   - 错误      Encode —— 序列化失败
 *
 * 参数：value — 待编码的值
 */
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    codec_inner::encode(value)
}

/**
 * 由边界字节解码为值。
 *
 * 契约：API-032
 *
 * 约束：
 *   - requires  T 实现序列化框架的对应特征
 *   - ensures   编解码只出现在字节通道；失败返回 Err
 *   - 错误      Decode —— 字节不合法或结构不符
 *
 * 参数：bytes — 边界字节
 */
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    codec_inner::decode(bytes)
}
