//! File and filesystem-related syscalls
use core::any::Any;
use core::mem::size_of;
use alloc::boxed::Box;

use crate::fs::{linkat, open_file, unlinkat, File, OSInode, OpenFlags, Stat};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    let task = current_task().unwrap();
    trace!(
        "kernel:pid[{}] sys_fstat",
        task.pid.0
    );

    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    };

    let stat: *const u8;
    if let Some(file) = &inner.fd_table[fd] {
        if let Some(file) = file.clone().as_any().downcast_ref::<OSInode>() {
            stat = Box::into_raw(Box::new(Stat {
                dev: 0,
                ino: file.get_inode_id() as u64,
                mode: file.get_inode_type(),
                nlink: file.get_nlink(),
                pad: [0u64; 7],
            })) as *const u8;
        } else {
            return -1;
        };
     } else {
        return -1;
    };
    drop(inner);

    let buffers = translated_byte_buffer(current_user_token(), st as *const u8, size_of::<Stat>());
    let mut index = 0;
    for buffer in buffers {
        for byte in buffer {
            *byte = unsafe { *stat.offset(index) };
            index += 1;
        };
    };
    0
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    let task = current_task().unwrap();
    trace!(
        "kernel:pid[{}] sys_linkat",
        task.pid.0
    );
    
    let token = current_user_token();
    let old_name = translated_str(token, old_name);
    let new_name = translated_str(token, new_name);

    if old_name == new_name {
        return -1;
    };

    linkat(old_name.as_str(), new_name.as_str())
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}
