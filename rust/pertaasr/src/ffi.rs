use sync_ptr::{SyncFnPtr, sync_fn_ptr_from_addr};

pub type GetAddressFnForVoidReturnType = extern "C" fn(i32);
pub type GetAddressFnForLongReturnType = extern "C" fn(i32) -> u64;

pub fn sync_fn_ptr_from_add_void(addr: usize) -> Option<SyncFnPtr<GetAddressFnForVoidReturnType>> {
    if addr == 0 {
        None
    } else {
        unsafe { Some(sync_fn_ptr_from_addr!(GetAddressFnForVoidReturnType, addr)) }
    }
}

pub fn sync_fn_ptr_from_add_u64(addr: usize) -> Option<SyncFnPtr<GetAddressFnForLongReturnType>> {
    if addr == 0 {
        None
    } else {
        unsafe { Some(sync_fn_ptr_from_addr!(GetAddressFnForLongReturnType, addr)) }
    }
}
