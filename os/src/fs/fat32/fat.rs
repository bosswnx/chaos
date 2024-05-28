use alloc::sync::Arc;

use crate::block::{block_cache::get_block_cache, block_dev::BlockDevice, BLOCK_SZ};

use super::super_block::Fat32SB;

pub struct FAT {
    pub start_sector: u32,
    pub sb: Arc<Fat32SB>,
    pub bdev: Arc<dyn BlockDevice>,
}

impl FAT {
    pub fn from_sb(sb: Arc<Fat32SB>, bdev: &Arc<dyn BlockDevice>) -> Self {
        Self {
            start_sector: sb.reserved_sectors_cnt as u32,
            sb,
            bdev: Arc::clone(bdev),
        }
    }

    /// allocate a new cluster
    pub fn alloc_new_cluster(&self, bdev: &Arc<dyn BlockDevice>) -> Option<usize> {
        let mut offset = self.start_sector * BLOCK_SZ as u32 + 3 * 4;
        let mut cluster_id = 0;
        loop {
            let fat_sector = offset / BLOCK_SZ as u32;
            let offset_in_sector = offset % BLOCK_SZ as u32;
            get_block_cache(fat_sector as usize, Arc::clone(bdev))
                .lock()
                .read(offset_in_sector as usize, |data: &[u8; 4]| {
                    let num = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    if num == 0 {
                        cluster_id = (offset - self.start_sector * BLOCK_SZ as u32) / 4;
                    }
                });
            if cluster_id != 0 {
                break;
            }
            offset += 4;
        }
        Some(cluster_id as usize)
    }
    
    /// get next cluster number
    pub fn next_cluster_id(&self, cluster: usize) -> Option<usize> {
        let fat_offset = self.start_sector as usize * BLOCK_SZ + cluster * 4;
        let fat_sector = fat_offset / BLOCK_SZ;
        let fat_offset_in_sector = fat_offset % BLOCK_SZ;
        let mut next_cluster = 0;
        get_block_cache(fat_sector as usize, Arc::clone(&self.bdev))
            .lock()
            .read(fat_offset_in_sector as usize, |data: &[u8; 4]| {
                next_cluster = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            });
        if next_cluster >= 0x0FFFFFF8 {
            return None;
        } else {
            return Some(next_cluster as usize);
        }
    }

     /// get next dentry sector id and offset
    pub fn next_dentry_id(&self, sector_id: usize, offset: usize) -> Option<(usize, usize)> {
        if offset >= 512 || offset % 32 != 0 {
            return None;
        }
        let next_offset = offset + 32;
        if next_offset >= 512 {
            let next_sector_id = sector_id + 1;
            if next_sector_id % self.sb.sectors_per_cluster as usize == 0 {
                if let Some(next_sector_id) = self.next_cluster_id(sector_id) {
                    Some((next_sector_id, 0))
                } else {
                    None
                }
            } else {
                Some((next_sector_id, 0))
            }
        } else {
            Some((sector_id, next_offset))
        }
    }

    #[allow(unused)]
    /// cluster id to sector id
    pub fn cluster_id_to_sector_id(&self, cluster: usize) -> Option<usize> {
        if cluster < 2 {
            return None;
        }
        let res = self.sb.root_sector() + (cluster - 2) * self.sb.sectors_per_cluster as usize;
        Some(res)
    }

    #[allow(unused)]
    /// sector id to cluster id
    pub fn sector_id_to_cluster_id(&self, sector: usize) -> Option<usize> {
        if sector < self.sb.root_sector() {
            return None;
        }
        let res = (sector - self.sb.root_sector()) / self.sb.sectors_per_cluster as usize + 2;
        Some(res)
    }
}

