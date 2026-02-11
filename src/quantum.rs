use std::arch::asm;
use std::ffi::c_void;
use crate::dynamo;

pub struct QuantumGate {
    ssn_delay: u16,
    ssn_query: u16,
    _ssn_alloc: u16,
    ssn_set_thread: u16,
    syscall_gadget: u64, 
}

impl QuantumGate {
    pub unsafe fn new() -> Option<Self> {
        if let Some(ntdll) = dynamo::Dynamo::get_module_base("ntdll.dll\0") {
            let ssn_delay = Self::resolve_ssn(ntdll, "NtDelayExecution\0")?;
            let ssn_query = Self::resolve_ssn(ntdll, "NtQuerySystemInformation\0")?;
            let ssn_alloc = Self::resolve_ssn(ntdll, "NtAllocateVirtualMemory\0")?;
            let ssn_set_thread = Self::resolve_ssn(ntdll, "NtSetInformationThread\0")?;
            
            let syscall_gadget = Self::find_syscall_gadget(ntdll)?;

            Some(QuantumGate { 
                ssn_delay,
                ssn_query,
                _ssn_alloc: ssn_alloc,
                ssn_set_thread,
                syscall_gadget
            })
        } else {
            None
        }
    }
    
    // ... (resolve_ssn / find_syscall_gadget logic omitted for brevity in tool call, just rewriting file to ensure consistency)
    // Wait, rewriting entire file is safer than patchy replace calls that fail.
    
    unsafe fn resolve_ssn(base: usize, func_name: &str) -> Option<u16> {
        let func_addr = dynamo::Dynamo::get_func_ptr(base, func_name)?;
        let ptr = func_addr as *const u8;
        // Check for Hooking (Is it E9 ...?)
        // Standard syscall stub:
        // 4C 8B D1 (mov r10, rcx)
        // B8 SSN 00 00 (mov eax, SSN)
        if *ptr == 0x4C && *ptr.add(1) == 0x8B && *ptr.add(2) == 0xD1 && *ptr.add(3) == 0xB8 {
            let high = *ptr.add(5);
            let low = *ptr.add(4);
            return Some(((high as u16) << 8) | low as u16);
        }
        // If hooked, scan neighbors? (For now, just return None or do minimal scan)
        // Simplistic approach: Just read it.
        if *ptr.add(3) == 0xB8 {
             let high = *ptr.add(5);
             let low = *ptr.add(4);
             return Some(((high as u16) << 8) | low as u16);
        }
        None
    }

    unsafe fn find_syscall_gadget(base: usize) -> Option<u64> {
        // Scan for 0F 05 C3 (syscall; ret)
        // Limit scan to .text section? Just scan first megabyte for now.
        let ptr = base as *const u8;
        for i in 0..0x100000 {
            if *ptr.add(i) == 0x0F && *ptr.add(i+1) == 0x05 && *ptr.add(i+2) == 0xC3 {
                return Some((base + i) as u64);
            }
        }
        None
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn quantum_sleep(&self, valid_handle: isize, delay_val: i64) -> u32 {
        let ssn = self.ssn_delay as u32;
        let gadget = self.syscall_gadget;
        let status: u32;
        let delay_ptr = &delay_val as *const i64;
    
        asm!(
            "mov r10, rcx",
            "mov eax, {0:e}",
            "call {1}",
            in(reg) ssn,
            in(reg) gadget,
            in("rcx") valid_handle, // Handle (optional/NULL)
            in("rdx") 0, // Alertable
            in("r8") delay_ptr, // Time
            lateout("rax") status,
            lateout("r10") _,
            lateout("r11") _
        );
        status
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn query_system_info(&self, info_class: u32, buf: *mut c_void, size: u32, ret_len: *mut u32) -> u32 {
        let ssn = self.ssn_query as u32;
        let gadget = self.syscall_gadget;
        let status: u32;
        
        asm!(
            "mov r10, rcx",
            "mov eax, {0:e}",
            "call {1}",
            in(reg) ssn,
            in(reg) gadget,
            in("rcx") info_class,
            in("rdx") buf,
            in("r8") size,
            in("r9") ret_len,
            lateout("rax") status,
            lateout("r10") _,
            lateout("r11") _
        );
        status
    }

    #[allow(dead_code)]
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn allocate_virtual_memory(&self, _process_handle: isize, _base_addr: *mut *mut c_void, _zero_bits: usize, _region_size: *mut usize, _alloc_type: u32, _protect: u32) -> u32 {
        // Placeholder to avoid unused variable warning and stable stack alignment issues
        let _ssn = self._ssn_alloc as u32;
        let _gadget = self.syscall_gadget;
        0 
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn set_information_thread(&self, thread_handle: isize, thread_info_class: u32, thread_info: *const c_void, thread_info_len: u32) -> u32 {
        let ssn = self.ssn_set_thread as u32;
        let gadget = self.syscall_gadget;
        let status: u32;
        
        asm!(
            "mov r10, rcx",
            "mov eax, {0:e}",
            "call {1}",
            in(reg) ssn,
            in(reg) gadget,
            in("rcx") thread_handle,
            in("rdx") thread_info_class,
            in("r8") thread_info,
            in("r9") thread_info_len,
            lateout("rax") status,
            lateout("r10") _,
            lateout("r11") _
        );
        status
    }
}

pub fn quantum_sleep(ms: u64) {
    unsafe {
        if let Some(gate) = QuantumGate::new() {
            let time = -10000 * (ms as i64);
            let _ = gate.quantum_sleep(0, time);
        } else {
             std::thread::sleep(std::time::Duration::from_millis(ms));
        }
    }
}
