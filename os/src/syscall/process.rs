//! Process management syscalls
use core::mem::size_of;
use alloc::boxed::Box;

use crate::{
    config::MAX_SYSCALL_NUM, mm::{translated_byte_buffer, MapPermission, PageTable, StepByOne, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task, get_task_start_time, get_task_syscall_times, suspend_current_and_run_next, task_alloc_mem, TaskStatus
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let buffers = translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());
    
    let us = get_time_us();
    let ts = Box::into_raw(Box::new(TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    })) as *const u8;

    let mut index = 0;
    for buffer in buffers {
        for byte in buffer {
            *byte = unsafe { *ts.offset(index) };
            index += 1;
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let buffers = translated_byte_buffer(current_user_token(), _ti as *const u8, size_of::<TaskInfo>());

    let id = get_current_task();
    let ti = Box::into_raw(Box::new(TaskInfo {
        status: TaskStatus::Running,
        syscall_times: get_task_syscall_times(id),
        time: get_time_ms() - get_task_start_time(id),
    })) as *const u8;

    let mut index = 0;
    for buffer in buffers {
        for byte in buffer {
            *byte = unsafe { *ti.offset(index) };
            index += 1;
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);

    // `_start` must align by page size
    if !start_va.aligned() {
        return -1;
    }

    // `_prot` must satisfy conditions
    if _prot & !0x7 != 0 || _prot & 0x7 == 0 {
        return -1;
    }

    let page_table = PageTable::from_token(current_user_token());
    let mut start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();

    while start_vpn < end_vpn {
        if let Some(pte) = page_table.translate(start_vpn) {
            if pte.is_valid() {
                // page already mapped
                return -1;
            }
        };

        start_vpn.step();
    };
    
    task_alloc_mem(get_current_task(), start_va, end_va, MapPermission::new((_prot as u8 + 0x8) << 1))
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
