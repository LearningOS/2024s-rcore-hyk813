### ch6练习

#### 零、支持前面的测例

只需要修改spawn即可。把获取数据，改成获取文件的数据。

#### 一、link

不太会写，直接先照抄讲解里的

- 为 inode 实现 link 和 unlink 方法

```Rust
/// easy-fs/src/vfs.rs

    /// Link
pub fn link(&self, oldname: &str, newname: &str) -> Option<()> {
    let mut fs = self.fs.lock();
    // First get old inode according to given `oldname`
    // Read from `DirEntry`
    let old_inode_id =
        self.read_disk_inode(|root_inode| self.find_inode_id(oldname, root_inode));
    if old_inode_id.is_none() {
        return None;
    }

    // Get position of old inode.
    let (block_id, block_offset) = fs.get_disk_inode_pos(old_inode_id.unwrap());

    // Get the target DiskInode according to `block_id` and `block_offset`
    get_block_cache(block_id as usize, Arc::clone(&self.block_device))
        .lock()
        // Increase the `nlink` of target DiskInode
        .modify(block_offset, |n: &mut DiskInode| n.nlink += 1);
    
    // Insert `newname` into directory.
    self.modify_disk_inode(|root_inode| {
        let file_count = (root_inode.size as usize) / DIRENT_SZ;
        let new_size = (file_count + 1) * DIRENT_SZ;
        self.increase_size(new_size as u32, root_inode, &mut fs);
        let dirent = DirEntry::new(newname, old_inode_id.unwrap());
        root_inode.write_at(
            file_count * DIRENT_SZ,
            dirent.as_bytes(),
            &self.block_device,
        );
    });

    // Since we may have writed the cached block, we need to flush the cache.
    block_cache_sync_all();
    Some(())
}
```

实现之后，我们因为只有一个目录。直接在根目录ROOT_INODE里面link就好了，无非是在哪里写这个link。

和之前一样，转到fs/inode.rs里面完成。特别地，link返回NONE则系统调用返回-1

unlink同理。

#### 二、fstat

通过fd，我们只能访问当前processor的fd_table，然后访问一个实现了File的trait的类型，这里特指OSINODE类型。

现在FILE实现stat方法。然后再OSINODE里面实现如下：

```rust
 /// os/src/fs/inode.rs
impl File for OSInode {
    fn stat(&self, st: &mut Stat) -> isize {
        let inner = self.inner.exclusive_access();
        inner.inode.read_disk_inode(|disk_inode| {
            st.mode = match disk_inode.type_ {
                DiskInodeType::File => StatMode::FILE,
                DiskInodeType::Directory => StatMode::DIR,
            };
            st.nlink = disk_inode.nlink;
        });
        0
    }
}
```

stat的ino好像没做要求，我也不会写。

