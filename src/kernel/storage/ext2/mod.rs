use alloc::{sync::Arc, vec::Vec};
use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;

const BLOCK_SIZE: usize = 512;

lazy_static! {
    static ref FILESYSTEM: Mutex<FileSystem<'static>> = Mutex::new(FileSystem::new(10, 100));
}

struct FileSystem<'a> {
    super_block: SuperBlock,
    head_block: Block,
    inodes: Vec<INode<'a>>,
}

impl FileSystem<'_> {
    pub fn new(inode_count: usize, block_count: usize) -> Self {
        Self {
            super_block: SuperBlock::new(inode_count, block_count),
            head_block: Block::new(),
            inodes: (0..inode_count).fold(vec![], |mut acc, _| {
                acc.push(INode::new("", 0));
                acc
            }),
        }
    }
}

// Metadata about the filesystem
struct SuperBlock {
    inode_count: usize,
    block_count: usize,
    block_size: usize,
}

impl SuperBlock {
    pub fn new(inode_count: usize, block_count: usize) -> Self {
        Self {
            inode_count,
            block_count,
            block_size: size_of::<Block>(),
        }
    }

    pub fn mount() -> Self {
        unimplemented!();
    }

    pub fn sync() -> Self {
        unimplemented!();
    }
}

struct Block {
    data: [u8; BLOCK_SIZE],
    next: Option<Arc<Block>>,
}

impl Block {
    pub fn new() -> Self {
        Self {
            data: [0; BLOCK_SIZE],
            next: None,
        }
    }
}

struct INode<'a> {
    name: &'a str,
    size: usize,
}

impl<'a> INode<'a> {
    pub fn new(name: &'a str, size: usize) -> Self {
        Self { name, size }
    }
}
