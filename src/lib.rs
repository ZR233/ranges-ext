#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{fmt::Debug, ops::Range};

/// RangeSet 操作 trait，为容器类型提供区间集合功能
pub trait RangeSetOps<T: RangeInfo> {
    /// 添加一个区间（会自动合并相邻区间）
    fn merge_add(&mut self, new_info: T, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>>;

    /// 删除一个区间
    fn merge_remove(
        &mut self,
        range: Range<T::Type>,
        temp_buffer: &mut [u8],
    ) -> Result<(), RangeError<T>>;

    /// 批量添加多个区间
    fn merge_extend<I>(&mut self, ranges: I, temp_buffer: &mut [u8]) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>;

    /// 查询某个值是否落在任意一个区间中
    fn merge_contains_point(&self, value: T::Type) -> bool;
}

/// RangeSet 操作 trait（alloc 版本），为带分配器的容器提供区间集合功能
/// 相比 RangeSetOps，不需要用户提供临时缓冲区
#[cfg(feature = "alloc")]
pub trait RangeSetAllocOps<T: RangeInfo> {
    /// 添加一个区间（会自动合并相邻区间）
    fn merge_add(&mut self, new_info: T) -> Result<(), RangeError<T>>;

    /// 删除一个区间
    fn merge_remove(&mut self, range: Range<T::Type>) -> Result<(), RangeError<T>>;

    /// 批量添加多个区间
    fn merge_extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T>>
    where
        I: IntoIterator<Item = T>;

    /// 查询某个值是否落在任意一个区间中
    fn merge_contains_point(&self, value: T::Type) -> bool;
}

/// RangeSet 错误类型
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RangeError<T>
where
    T: RangeInfo,
{
    /// 容量不足错误
    #[error("RangeSet capacity exceeded")]
    Capacity,
    /// 区间冲突错误：尝试覆盖不可覆盖的区间
    #[error("Range conflict: new {new:?} conflicts with existing non-overwritable {existing:?}")]
    Conflict {
        /// 新添加的区间
        new: T,
        /// 已存在的冲突区间
        existing: T,
    },
}

pub trait RangeInfo: Debug + Clone + Sized + Default {
    type Kind: Debug + Eq + Clone;
    type Type: Ord + Copy;
    fn range(&self) -> Range<Self::Type>;
    fn kind(&self) -> Self::Kind;
    fn overwritable(&self) -> bool;
    fn clone_with_range(&self, range: Range<Self::Type>) -> Self;
}

mod core_ops;
/// 辅助函数模块
mod helpers;

#[cfg(feature = "alloc")]
mod alloc_ops;
/// 实现模块
mod heapless_ops;
