//! Stack guard for the recursive-descent PEG parser.
//!
//! Deep input (or a left-recursive rule) makes the parser recurse deeply.
//! Instead of overflowing the thread's stack, [`Peg::parse`] fails with
//! [`ErrorKind::RecursionLimit`] once the stack pointer gets close to the
//! end of the current thread's stack. Where the stack bounds can be queried
//! (Linux glibc/musl, macOS) the whole stack minus a safety margin is used;
//! elsewhere a fixed [`FALLBACK_STACK_BUDGET`] is allowed.
//!
//! [`Peg::parse`]: crate::peg::grammar::Peg::parse
//! [`ErrorKind::RecursionLimit`]: crate::peg::parsing::ErrorKind::RecursionLimit

use std::cell::Cell;

/// Stack bytes a parse may use when the thread's stack size is unknown:
/// comfortably below the 2 MiB of a default spawned Rust thread.
pub const FALLBACK_STACK_BUDGET: usize = 1024 * 1024;

/// Head room kept free below the guard: stack used between two checks (one
/// nonterminal level) plus whatever the caller does after the parse fails.
const SAFETY_MARGIN: usize = 256 * 1024;

/// Current stack address (stacks grow downward on all supported targets).
#[inline(always)]
pub(crate) fn stack_address() -> usize {
    let probe = 0u8;
    std::hint::black_box(&probe) as *const u8 as usize
}

/// Lowest address the parse may reach, given an optional byte budget.
pub(crate) fn stack_floor(budget: Option<usize>) -> usize {
    let here = stack_address();
    let by_budget = |bytes: usize| here.saturating_sub(bytes).max(1);
    match (thread_stack_low(), budget) {
        (Some(low), budget) => {
            // Small stacks keep a proportionally smaller margin.
            let margin = SAFETY_MARGIN.min(here.saturating_sub(low) / 4);
            let by_stack = low.saturating_add(margin).max(1);
            match budget {
                Some(bytes) => by_stack.max(by_budget(bytes)),
                None => by_stack,
            }
        }
        (None, Some(bytes)) => by_budget(bytes.min(FALLBACK_STACK_BUDGET)),
        (None, None) => by_budget(FALLBACK_STACK_BUDGET),
    }
}

thread_local! {
    /// Lowest address of this thread's stack (0 = not looked up yet,
    /// 1 = unknown).
    static STACK_LOW: Cell<usize> = const { Cell::new(0) };
}

fn thread_stack_low() -> Option<usize> {
    let cached = STACK_LOW.try_with(Cell::get).unwrap_or(1);
    let low = if cached == 0 {
        let low = query_stack_low()
            .filter(|&low| low < stack_address())
            .unwrap_or(1);
        let _ = STACK_LOW.try_with(|c| c.set(low));
        low
    } else {
        cached
    };
    (low > 1).then_some(low)
}

#[cfg(all(target_os = "linux", any(target_env = "gnu", target_env = "musl")))]
fn query_stack_low() -> Option<usize> {
    // Opaque `pthread_attr_t`, larger than any libc's (56-64 bytes).
    #[repr(C, align(16))]
    struct PthreadAttr([u8; 256]);
    extern "C" {
        fn pthread_self() -> usize;
        fn pthread_getattr_np(thread: usize, attr: *mut PthreadAttr) -> i32;
        fn pthread_attr_getstack(
            attr: *const PthreadAttr,
            addr: *mut *mut u8,
            size: *mut usize,
        ) -> i32;
        fn pthread_attr_destroy(attr: *mut PthreadAttr) -> i32;
    }
    let mut attr = PthreadAttr([0; 256]);
    let mut addr: *mut u8 = std::ptr::null_mut();
    let mut size = 0usize;
    // SAFETY: `attr` is a writable buffer at least as large and aligned as
    // `pthread_attr_t`; it is initialised by `pthread_getattr_np` before use
    // and destroyed afterwards. `addr`/`size` are valid out-pointers.
    unsafe {
        if pthread_getattr_np(pthread_self(), &mut attr) != 0 {
            return None;
        }
        let ok = pthread_attr_getstack(&attr, &mut addr, &mut size) == 0;
        pthread_attr_destroy(&mut attr);
        (ok && !addr.is_null() && size > 0).then_some(addr as usize)
    }
}

#[cfg(target_os = "macos")]
fn query_stack_low() -> Option<usize> {
    extern "C" {
        fn pthread_self() -> usize;
        fn pthread_get_stackaddr_np(thread: usize) -> *mut u8;
        fn pthread_get_stacksize_np(thread: usize) -> usize;
    }
    // SAFETY: plain queries about the calling thread.
    let (top, size) = unsafe {
        let me = pthread_self();
        (
            pthread_get_stackaddr_np(me) as usize,
            pthread_get_stacksize_np(me),
        )
    };
    (top > size && size > 0).then(|| top - size)
}

#[cfg(not(any(
    all(target_os = "linux", any(target_env = "gnu", target_env = "musl")),
    target_os = "macos"
)))]
fn query_stack_low() -> Option<usize> {
    None
}
