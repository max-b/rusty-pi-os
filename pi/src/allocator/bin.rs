use std::fmt;
use alloc::alloc::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::{LinkedList, Node};

#[derive(Copy, Clone, Debug, Default)]
pub struct Bin {
    size: usize,
    count: usize,
    freed: LinkedList
}

/// MAX_BIN_SIZE is 1GB
const MAX_BIN_SIZE: usize = 1 << 39;

///  39 bins will cover 2^1 -> 2^39
const NUM_BINS: usize = 39;

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    bins: [Bin; NUM_BINS],
    current: usize,
    end: usize,
}

/// Returns the bit digit of the next power of two
fn power_of_two_digit(num: usize) -> usize {
    let mut cursor = num;
    let mut highest_bit_location = 0;
    while cursor > 0 {
        cursor = cursor >> 1;
        highest_bit_location += 1;
    }

    if num.is_power_of_two() {
        highest_bit_location
    } else {
        highest_bit_location + 1
    }
}

fn bin_num(size: usize) -> usize {
    power_of_two_digit(size).saturating_sub(1)
}

fn find_aligned_node(list: &mut LinkedList, align: usize) -> Option<Node> {
    for node in list.iter_mut() {
        if node.value() as usize % align == 0 {
            return Some(node);
        }
    }
    None
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let mut bins: [Bin; NUM_BINS] = [Default::default(); NUM_BINS];

        for i in 0..bins.len() {
            bins[i].size = 1 << i;
        }

        Allocator {
            current: start,
            end,
            bins
        }
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let size = layout.size();
        let align = layout.align();
        let bin_num = bin_num(size);
        let bin = &mut self.bins[bin_num];

        let aligned_node = find_aligned_node(&mut bin.freed, align);
        match aligned_node {
            Some(node) => {
                let val = node.pop() as *mut u8;
                bin.count -= 1;
                return Ok(val);
            }
            None => {}
        }

        // TODO: This should probably put the other freed node
        // into the current bin
        let mut bin_cursor = bin_num + 1;
        while bin_cursor < NUM_BINS {
            let aligned_node = find_aligned_node(&mut self.bins[bin_cursor].freed, align);
            match aligned_node {
                Some(node) => {
                    let val = node.pop() as *mut u8;
                    self.bins[bin_cursor].count -= 1;
                    return Ok(val);
                },
                None => {}
            }
            bin_cursor += 1;
        }

        let next_aligned = align_up(self.current, align);
        if next_aligned.saturating_add(size) >= self.end {
            return Err(AllocErr);
        }

        self.current = next_aligned.saturating_add(size);

        return Ok(next_aligned as *mut u8);
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let bin_num = bin_num(layout.size());
        let bin = &mut self.bins[bin_num];
        if layout.size() >= std::mem::size_of::<usize>() {
            unsafe {
                bin.freed.push(ptr as *mut usize);
                bin.count += 1;
            }
        }
    }
}

impl fmt::Debug for Allocator{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Current: {} End: {}\nBins: {:#x?}", self.current, self.end, &self.bins[..])
    }
}
