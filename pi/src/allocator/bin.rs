use std::{fmt, cmp};
use alloc::alloc::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::{LinkedList, Node};

#[derive(Copy, Clone, Debug, Default)]
pub struct Bin {
    size: usize,
    freed: LinkedList
}

///  39 bins will cover 2^1 -> 2^39(1GB)
const NUM_BINS: usize = 39;

/// Align to at least 0x10
const MIN_ALIGN: usize = 0x10;

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

fn calculate_bin_num(size: usize) -> usize {
    power_of_two_digit(size).saturating_sub(1)
}

fn find_aligned_node(list: &mut LinkedList, align: usize) -> Option<Node> {
    for node in list.iter_mut() {
        if ((node.value() as usize).trailing_zeros()) >= align.trailing_zeros() {
            return Some(node);
        }
    }
    None
}

fn find_containing_node(list: &mut LinkedList, align: usize, alloc_size: usize, bin_size: usize) -> Option<(Node, usize)> {
    let mask = (!0x00usize) << power_of_two_digit(align);
    for node in list.iter_mut() {
        let val = node.value() as usize;
        let end = val + bin_size;
        let next_aligned = (val.saturating_add(align)) & mask;
        if next_aligned > val && next_aligned + alloc_size <= end {
            return Some((node, next_aligned));
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
        let bin_num = calculate_bin_num(size);
        let bin_size = 1 << bin_num;
        let bin = &mut self.bins[bin_num];

        let aligned_node = find_aligned_node(&mut bin.freed, align);
        match aligned_node {
            Some(node) => {
                let val = node.pop() as *mut u8;
                return Ok(val);
            }
            None => {}
        }

        let mut bin_cursor = bin_num + 1;
        while bin_cursor < NUM_BINS {
            let aligned_node = find_aligned_node(&mut self.bins[bin_cursor].freed, align);
            match aligned_node {
                Some(node) => {
                    let val = node.pop() as *mut u8;
                    let new_segment_start = (val as usize).saturating_add(bin_size);
                    let cursor_bin_size = 1 << bin_cursor;
                    let new_segment_end = (val as usize).saturating_add(cursor_bin_size);
                    self.return_to_freed(new_segment_start, new_segment_end);

                    return Ok(val);
                },
                None => {}
            }
            bin_cursor += 1;
        }

        let mut bin_cursor = bin_num + 1;
        while bin_cursor < NUM_BINS {
            let containing_node = find_containing_node(&mut self.bins[bin_cursor].freed, align, size, 1 << bin_cursor);
            match containing_node {
                Some((node, next_aligned)) => {
                    let val = node.pop() as usize;
                    let cursor_bin_size = 1 << bin_cursor;
                    let new_segment_start = val as usize;
                    let new_segment_end = next_aligned;
                    self.return_to_freed(new_segment_start, new_segment_end);

                    let new_segment_start = next_aligned.saturating_add(bin_size);
                    let new_segment_end = val.saturating_add(cursor_bin_size);
                    self.return_to_freed(new_segment_start, new_segment_end);

                    return Ok(next_aligned as *mut u8);
                },
                None => {}
            }
            bin_cursor += 1;
        }

        let next_aligned = align_up(self.current, cmp::max(MIN_ALIGN, align));
        if next_aligned.saturating_add(size) >= self.end {
            return Err(AllocErr);
        }

        self.current = next_aligned.saturating_add(size);

        return Ok(next_aligned as *mut u8);
    }

    fn return_to_freed(&mut self, start: usize, end: usize) {
        unsafe { self.dealloc(start as *mut u8, Layout::from_size_align_unchecked(end.saturating_sub(start), 0)); }
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
        let mut bin_num = calculate_bin_num(layout.size());
        // If size does not exactly match bin size
        // can we put remainder into smaller bins?
        if !layout.size().is_power_of_two() {
            bin_num = bin_num.saturating_sub(1);
        }

        let bin = &mut self.bins[bin_num];
        let bin_size = 1 << bin_num;
        if layout.size() >= std::mem::size_of::<usize>() {
            // Check for segments we may be able to merge together
            for node in bin.freed.iter_mut() {
                let node_val = node.value() as usize;
                let ptr_val = ptr as usize;
                if ptr_val > node_val && node_val + bin_size == ptr_val {
                    node.pop();
                    unsafe { self.dealloc(node_val as *mut u8, Layout::from_size_align_unchecked(1 << (bin_num + 1), 0)); }
                    return;
                } else if node_val > ptr_val && ptr_val + bin_size == node_val {
                    node.pop();
                    unsafe { self.dealloc(ptr_val as *mut u8, Layout::from_size_align_unchecked(1 << (bin_num + 1), 0)); }
                    return;
                }
            }

            unsafe {
                bin.freed.push(ptr as *mut usize);
            }
        }
    }
}

impl fmt::Debug for Allocator{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Current: {} End: {}\nBins: {:#x?}", self.current, self.end, &self.bins[..])
    }
}
