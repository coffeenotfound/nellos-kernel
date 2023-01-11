//! Physical memory buddy allocator
//! 
//! We have buddies from the largest page size (4 MiB, 2^22) down to single page size (4 KiB, 2^12).
//! This is what linux uses (I think) and should be a good balance between the number of
//! buddy levels that need to be updated to allocate physical memory and efficiency of big allocations.
//! This also means the biggest chunk of physical memory we can efficiently allocate is 4 MiB.
//! This should (hopefully) be fine as most allocations will be paged anyway and then it doesn't matter
//! too much if the physical memory blocks are contiguous.
//! 
//! Note: Right now I implement a very inefficient dumb buddy allocator, but it'll work *shrug*

use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::AtomicU64;

use crate::cpu::CpuUid;
use crate::Phys;

pub const BUCK_UPPER_TREE_MAX_LEVEL: usize = 11;
pub const BUCK_LOWER_TREE_MAX_LEVEL: usize = 5;

pub const BASE_PAGE_ADDR_BITS: usize = 12;
pub const BASE_PAGE_SIZE: usize = (0x1 << BASE_PAGE_ADDR_BITS);

/// Platform base page page (as opposed to huge or giga pages)
#[repr(C, align(4096))]
pub struct BasePage([u64; 4096/8]);

/// A 2^12 page big buck allocator shard
///
/// Allocation must only be done from this shard's owning cpu
/// and has thus no synchronization
///
/// Freeing is harder since memory allocated by this shard's
/// cpu can be freed from another cpu. Because of that we store
/// atomic u64 that represent which blocks to be freed on the next
/// malloc (which will always happen on the owning cpu)
#[repr(C)]
pub struct BuckShard {
	/// A packed 6-high binary tree where each node is one bit.
	/// Tree root is bit zero
	upper_tree: u64,
	/// Each bit indicates whether the associated lower tree
	/// is allocated in full by a bigger allocation in the upper tree.
	lower_trees_alloced_by_upper: u64,
	/// Array indexed by lower tree layers where each bit indicates
	/// whether the associated level in the associated lower tree
	/// still has at least one block available.
	lower_levels_available: [u64; 6],
	lower_trees: [u64; 64],
	
	/// Note: *mut BasePage instead of NonNull<BasePage> since the
	/// first shard may start at physical address zero (even if it is
	/// partially cut to size.
	base_addr: Phys<*mut BasePage>,
	/// TODO: Maybe we want shards to be owned by cpus and by isrs if they need to do any allocation
	owning_cpu: CpuUid,
	
	free_in_upper: AtomicU64,
	free_lowers: AtomicU64,
	free_in_lowers: [AtomicU64; 64],
}

impl BuckShard {
	pub unsafe fn alloc(&mut self, level: usize) -> Option<BuckBlock> {
		// Check level upper bound
		if level > BUCK_UPPER_TREE_MAX_LEVEL {
			return None;
		}
		
		// Allocate in upper tree
		if level > BUCK_LOWER_TREE_MAX_LEVEL {
//			// DEBUG:
//			println!("ALLOC IN UPPER");
			
			// Alloc in upper tree
			let (upper_idx, upper_idx_in_level) = alloc_in_tree(&mut self.upper_tree, level - (BUCK_LOWER_TREE_MAX_LEVEL + 1), &mut false)?;
			
			// Mark all lower trees which are children of the alloced upper block as fully alloced
			self.lower_trees_alloced_by_upper |= UPPER_ALLOC_IDX_TO_LOWER_TREE_USED_FLAGS[upper_idx];
			
			// Return allocated block
			let block_addr = (self.base_addr as usize) + (upper_idx_in_level << (BASE_PAGE_ADDR_BITS + level));
			let block_size = (BASE_PAGE_SIZE << level) as u32;
			
			// Assert
			debug_assert!(
				block_addr != 0 && (block_addr & ((0x1 << BASE_PAGE_ADDR_BITS) - 1)) == 0,
				"Buck block addr cannot be null and must be divisible by the page size",
			);
			
			return Some(BuckBlock {
				level: level as u32,
				ptr: Phys(NonNull::new_unchecked(block_addr as *mut BasePage)),
				real_size: block_size,
			});
		}
		// Allocate in one of the lower trees
		else {
//			// DEBUG:
//			println!("ALLOC IN LOWER");
			
			// Bitfield of lower tree of whether they have at least one
			// block of the requested level available
			// (which includes not being in use by an upper tree allocation!)
			let trees_with_blocks_avail = (self.lower_levels_available[level] & !self.lower_trees_alloced_by_upper);
			
//			// DEBUG:
//			println!("trees_with_blocks_avail {trees_with_blocks_avail:064b}");
			
			// Find first suitable tree
			let lower_tree_idx = trees_with_blocks_avail.trailing_zeros() as usize;
			
//			// DEBUG:
//			println!("lower_tree_idx {lower_tree_idx}");
			
			// No suitably free lower tree available
			if lower_tree_idx == 64 {
				return None;
			}
			
			// Allocate in lower tree
			let lower_tree_ref = unsafe { self.lower_trees.get_unchecked_mut(lower_tree_idx) };
			
			let mut is_now_full = false;
			let (_, block_idx_in_level) = alloc_in_tree(lower_tree_ref, level, &mut is_now_full)?;
			
			// If level is now full, remove from available flags
			if is_now_full {
				self.lower_levels_available[level] &= !(0x1 << lower_tree_idx);
			}
			
			// Mark upper tree parents as used
			let mut upper_tree_new_bits = 0;
			let mut upper_parent_idx = (lower_tree_idx >> 1) + 31;
			
//			// DEBUG:
//			println!("upper_parent_idx {upper_parent_idx}");
			
			for _ in 0..6 {
				upper_tree_new_bits |= (0x1 << upper_parent_idx);
				upper_parent_idx = (upper_parent_idx - 1) >> 1;
			}
			self.upper_tree |= upper_tree_new_bits;
			
//			// DEBUG:
//			println!("block_idx {block_idx}");
			
			// Return allocated block
			let block_addr = (self.base_addr as usize) + (block_idx_in_level << (BASE_PAGE_ADDR_BITS + level));
			let block_size = (BASE_PAGE_SIZE << level) as u32;
			
			return Some(BuckBlock {
				level: level as u32,
				ptr: Phys(NonNull::new_unchecked(block_addr as *mut BasePage)),
				real_size: block_size,
			});
		}
	}
	
	/// Actually processes queued frees.
	/// Must only be called by the shard's owning cpu
	pub unsafe fn free_local(&mut self, block: BuckBlock) {
		todo!()
	}
	
	pub const fn new() -> Self {
		const ATOMIC_ZERO: AtomicU64 = AtomicU64::new(0);
		Self {
			upper_tree: 0,
			lower_trees: [0; 64],
			lower_trees_alloced_by_upper: 0,
			lower_levels_available: [u64::MAX; 6],
			
			base_addr: Phys(ptr::null_mut()),
			owning_cpu: CpuUid(0),
			
			free_in_upper: ATOMIC_ZERO,
			free_lowers: ATOMIC_ZERO,
			free_in_lowers: [ATOMIC_ZERO; 64],
		}
	}
}

/// NOTE: Level goes from 5 (biggest) to 0 (smallest)
/// Returns an index not a bitfield!
#[inline(always)]
fn alloc_in_tree(tree: &mut u64, level: usize, is_now_full: &mut bool) -> Option<(usize, usize)> {
	let level_shift = (32 >> level) - 1;
	let highest_idx_in_level = level_shift;
	
	// TODO: trailing_ones() can likely be optimized by at least one branch
	//  by doing an unchecked clz with `tree | highest_u64_bit`.
	//  The highest bit is free anyways and the logic of checking > highest_idx_in_level still works out
	let free_block_idx_in_level = (*tree >> level_shift).trailing_ones() as usize;
	
	// No free block in level
	if free_block_idx_in_level > highest_idx_in_level {
		return None;
	}
	
	*is_now_full = (free_block_idx_in_level == highest_idx_in_level);
	
	let free_block_idx = free_block_idx_in_level + level_shift;

//	println!("Free block idx {free_block_idx}");
	
	// Mark self and children as used
	let mark_self_and_children = if free_block_idx < 31 {
		MARK_SELF_AND_CHILDREN_LUT[free_block_idx]
	} else {
		// We're in the lowest level, so just mark the block
		// itself as used
		(0x1 << free_block_idx)
	};
	let mut upper_tree_new_bits = mark_self_and_children;
	
	// Mark parents as used
	let mut parent_idx = free_block_idx;
	for _ in 0..5 {
		parent_idx = (parent_idx - 1) >> 1;
		upper_tree_new_bits |= (0x1 << parent_idx);
	}
	*tree |= upper_tree_new_bits;
	
	return Some((free_block_idx, free_block_idx_in_level));
}

/// An allocated buck allocator physical memory block
#[derive(Clone, Debug)]
#[repr(C)]
pub struct BuckBlock {
	ptr: Phys<NonNull<BasePage>>,
	/// The real size of this buddy block in bytes
	/// (not the requested allocation size but the actual block size)
	real_size: u32,
	/// The buddy order of this block, starting at zero for the smallest block size
	level: u32,
}

unsafe impl Send for BuckBlock {}
unsafe impl Sync for BuckBlock {}

pub enum BuddyAllocErr {
	NoneAvailable,
}

const MARK_SELF_AND_CHILDREN_LUT: [u64; 31] = [
	0b11111111111111111111111111111111_1111111111111111_11111111_1111_11_1,
	0b00000000000000001111111111111111_0000000011111111_00001111_0011_01_0,
	0b11111111111111110000000000000000_1111111100000000_11110000_1100_10_0,
	0b00000000000000000000000011111111_0000000000001111_00000011_0001_00_0,
	0b00000000000000001111111100000000_0000000011110000_00001100_0010_00_0,
	0b00000000111111110000000000000000_0000111100000000_00110000_0100_00_0,
	0b11111111000000000000000000000000_1111000000000000_11000000_1000_00_0,
	0b00000000000000000000000000001111_0000000000000011_00000001_0000_00_0,
	0b00000000000000000000000011110000_0000000000001100_00000010_0000_00_0,
	0b00000000000000000000111100000000_0000000000110000_00000100_0000_00_0,
	0b00000000000000001111000000000000_0000000011000000_00001000_0000_00_0,
	0b00000000000011110000000000000000_0000001100000000_00010000_0000_00_0,
	0b00000000111100000000000000000000_0000110000000000_00100000_0000_00_0,
	0b00001111000000000000000000000000_0011000000000000_01000000_0000_00_0,
	0b11110000000000000000000000000000_1100000000000000_10000000_0000_00_0,
	0b00000000000000000000000000000011_0000000000000001_00000000_0000_00_0,
	0b00000000000000000000000000001100_0000000000000010_00000000_0000_00_0,
	0b00000000000000000000000000110000_0000000000000100_00000000_0000_00_0,
	0b00000000000000000000000011000000_0000000000001000_00000000_0000_00_0,
	0b00000000000000000000001100000000_0000000000010000_00000000_0000_00_0,
	0b00000000000000000000110000000000_0000000000100000_00000000_0000_00_0,
	0b00000000000000000011000000000000_0000000001000000_00000000_0000_00_0,
	0b00000000000000001100000000000000_0000000010000000_00000000_0000_00_0,
	0b00000000000000110000000000000000_0000000100000000_00000000_0000_00_0,
	0b00000000000011000000000000000000_0000001000000000_00000000_0000_00_0,
	0b00000000001100000000000000000000_0000010000000000_00000000_0000_00_0,
	0b00000000110000000000000000000000_0000100000000000_00000000_0000_00_0,
	0b00000011000000000000000000000000_0001000000000000_00000000_0000_00_0,
	0b00001100000000000000000000000000_0010000000000000_00000000_0000_00_0,
	0b00110000000000000000000000000000_0100000000000000_00000000_0000_00_0,
	0b11000000000000000000000000000000_1000000000000000_00000000_0000_00_0,
];

const UPPER_ALLOC_IDX_TO_LOWER_TREE_USED_FLAGS: [u64; 63] = [
	0b1111111111111111111111111111111111111111111111111111111111111111,
	0b0000000000000000000000000000000011111111111111111111111111111111,
	0b1111111111111111111111111111111100000000000000000000000000000000,
	0b0000000000000000000000000000000000000000000000001111111111111111,
	0b0000000000000000000000000000000011111111111111110000000000000000,
	0b0000000000000000111111111111111100000000000000000000000000000000,
	0b1111111111111111000000000000000000000000000000000000000000000000,
	0b0000000000000000000000000000000000000000000000000000000011111111,
	0b0000000000000000000000000000000000000000000000001111111100000000,
	0b0000000000000000000000000000000000000000111111110000000000000000,
	0b0000000000000000000000000000000011111111000000000000000000000000,
	0b0000000000000000000000001111111100000000000000000000000000000000,
	0b0000000000000000111111110000000000000000000000000000000000000000,
	0b0000000011111111000000000000000000000000000000000000000000000000,
	0b1111111100000000000000000000000000000000000000000000000000000000,
	0b0000000000000000000000000000000000000000000000000000000000001111,
	0b0000000000000000000000000000000000000000000000000000000011110000,
	0b0000000000000000000000000000000000000000000000000000111100000000,
	0b0000000000000000000000000000000000000000000000001111000000000000,
	0b0000000000000000000000000000000000000000000011110000000000000000,
	0b0000000000000000000000000000000000000000111100000000000000000000,
	0b0000000000000000000000000000000000001111000000000000000000000000,
	0b0000000000000000000000000000000011110000000000000000000000000000,
	0b0000000000000000000000000000111100000000000000000000000000000000,
	0b0000000000000000000000001111000000000000000000000000000000000000,
	0b0000000000000000000011110000000000000000000000000000000000000000,
	0b0000000000000000111100000000000000000000000000000000000000000000,
	0b0000000000001111000000000000000000000000000000000000000000000000,
	0b0000000011110000000000000000000000000000000000000000000000000000,
	0b0000111100000000000000000000000000000000000000000000000000000000,
	0b1111000000000000000000000000000000000000000000000000000000000000,
	0b0000000000000000000000000000000000000000000000000000000000000011,
	0b0000000000000000000000000000000000000000000000000000000000001100,
	0b0000000000000000000000000000000000000000000000000000000000110000,
	0b0000000000000000000000000000000000000000000000000000000011000000,
	0b0000000000000000000000000000000000000000000000000000001100000000,
	0b0000000000000000000000000000000000000000000000000000110000000000,
	0b0000000000000000000000000000000000000000000000000011000000000000,
	0b0000000000000000000000000000000000000000000000001100000000000000,
	0b0000000000000000000000000000000000000000000000110000000000000000,
	0b0000000000000000000000000000000000000000000011000000000000000000,
	0b0000000000000000000000000000000000000000001100000000000000000000,
	0b0000000000000000000000000000000000000000110000000000000000000000,
	0b0000000000000000000000000000000000000011000000000000000000000000,
	0b0000000000000000000000000000000000001100000000000000000000000000,
	0b0000000000000000000000000000000000110000000000000000000000000000,
	0b0000000000000000000000000000000011000000000000000000000000000000,
	0b0000000000000000000000000000001100000000000000000000000000000000,
	0b0000000000000000000000000000110000000000000000000000000000000000,
	0b0000000000000000000000000011000000000000000000000000000000000000,
	0b0000000000000000000000001100000000000000000000000000000000000000,
	0b0000000000000000000000110000000000000000000000000000000000000000,
	0b0000000000000000000011000000000000000000000000000000000000000000,
	0b0000000000000000001100000000000000000000000000000000000000000000,
	0b0000000000000000110000000000000000000000000000000000000000000000,
	0b0000000000000011000000000000000000000000000000000000000000000000,
	0b0000000000001100000000000000000000000000000000000000000000000000,
	0b0000000000110000000000000000000000000000000000000000000000000000,
	0b0000000011000000000000000000000000000000000000000000000000000000,
	0b0000001100000000000000000000000000000000000000000000000000000000,
	0b0000110000000000000000000000000000000000000000000000000000000000,
	0b0011000000000000000000000000000000000000000000000000000000000000,
	0b1100000000000000000000000000000000000000000000000000000000000000,
];

/*
#![allow(unused_parens)]

use std::sync::atomic::*;

pub fn main() {
	unsafe {
		let mut shard = BuckShard::new();
		draw(shard.upper_tree);
		shard.alloc(0);
		draw(shard.upper_tree);
		shard.alloc(0);
		draw(shard.upper_tree);
		shard.alloc(7);
		draw(shard.upper_tree);
		shard.alloc(9);
		draw(shard.upper_tree);
	}
}

pub fn draw(tree: u64) {
    let mut cursor: usize = 0;
    for l in 0..6 {
        print!("L{} ", 5-l);
        for _ in 0..(0x1 << l) {
            if ((tree >> cursor) & 0x1) != 0 {
                print!("x");
            } else {
                print!(".");
            }
            cursor += 1;
        }
        println!();
    }
}

// ---------
*/