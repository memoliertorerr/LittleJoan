//! macOS: App Nap and a summon (LJ19.1). With no window, Little Joan is an invisible accessory
//! app, which macOS may put into App Nap after a while: lower CPU priority, coalesced timers and
//! throttled disk reads — just when the first summon after a long idle has to read the app's and
//! WebKit's code back from disk. So a summon that builds the window holds a user-initiated,
//! latency-critical activity (`NSProcessInfo`) until the window is up (`SummonActivity`, dropped by
//! joan::run). Never at rest: nothing holds one while hibernating, so App Nap saves energy as
//! before.
//!
//! For the smoke log only, `nap_state` reads the task's suppression policy (App Nap's mechanism in
//! the kernel: `task_policy_get` with `TASK_SUPPRESSION_POLICY` on the app's own task), so CI can
//! see whether a summon came in while napping.

use objc2::msg_send;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::NSString;

/// `NSActivityUserInitiatedAllowingIdleSystemSleep` (0x00EF_FFFF: user-initiated, without keeping
/// the system awake) | `NSActivityLatencyCritical` (0xFF_0000_0000: timers and I/O at full
/// precision), from Foundation's NSProcessInfo.h.
const OPTIONS: u64 = 0x00EF_FFFF | 0xFF_0000_0000;

/// A summon's activity: begun with `begin`, ended when dropped.
pub struct SummonActivity {
    info: Retained<AnyObject>,
    token: Retained<AnyObject>,
}

// SAFETY: NSProcessInfo is thread-safe, and an activity's token is an opaque object that
// -endActivity: takes on any thread; nothing else touches either.
unsafe impl Send for SummonActivity {}

impl SummonActivity {
    /// Begins the activity, or `None` if Foundation refused (the summon goes on without it).
    /// Called off the main thread (joan::run), so it brings its own autorelease pool.
    pub fn begin() -> Option<SummonActivity> {
        let class = AnyClass::get(c"NSProcessInfo")?;
        autoreleasepool(|_| {
            let reason = NSString::from_str("Little Joan is opening its window");
            // SAFETY: +processInfo returns the shared NSProcessInfo (or nil); the selector
            // -beginActivityWithOptions:reason: takes NSActivityOptions (a uint64_t) and an
            // NSString and returns the activity's token.
            unsafe {
                let info: Option<Retained<AnyObject>> = msg_send![class, processInfo];
                let info = info?;
                let token: Option<Retained<AnyObject>> =
                    msg_send![&*info, beginActivityWithOptions: OPTIONS, reason: &*reason];
                Some(SummonActivity {
                    info,
                    token: token?,
                })
            }
        })
    }
}

impl Drop for SummonActivity {
    fn drop(&mut self) {
        autoreleasepool(|_| {
            // SAFETY: -endActivity: takes the token -beginActivityWithOptions:reason: returned.
            unsafe {
                let _: () = msg_send![&*self.info, endActivity: &*self.token];
            }
        });
    }
}

/// `TASK_SUPPRESSION_POLICY` in <mach/task_policy.h>.
const TASK_SUPPRESSION_POLICY: u32 = 3;
/// `TASK_SUPPRESSION_POLICY_COUNT`: `struct task_suppression_policy` is 16 `integer_t`s (`active`,
/// `lowpri_cpu`, `timer_throttle`, `disk_throttle`, `cpu_limit`, `suspend`, `throughput_qos`,
/// `suppressed_cpu`, `background_sockets`, then 7 reserved).
const SUPPRESSION_COUNT: u32 = 16;

unsafe extern "C" {
    /// The task port of this process (`mach_task_self()` is a macro for it).
    static mach_task_self_: u32;
    fn task_policy_get(
        task: u32,
        flavor: u32,
        policy_info: *mut i32,
        count: *mut u32,
        get_default: *mut u32,
    ) -> i32;
}

/// Smoke mode: App Nap's state of this process now — `active=<0|1> lowpri=<n> timer=<n>
/// disk=<n>` from its suppression policy — or `None` when the kernel doesn't say.
pub fn nap_state() -> Option<String> {
    let mut info = [0_i32; SUPPRESSION_COUNT as usize];
    let mut count = SUPPRESSION_COUNT;
    let mut get_default = 0_u32;
    // SAFETY: task_policy_get writes at most `count` integers into `info` (which holds that
    // many) and updates `count`; the task is this process's own port, read from libSystem's
    // global.
    let result = unsafe {
        task_policy_get(
            mach_task_self_,
            TASK_SUPPRESSION_POLICY,
            info.as_mut_ptr(),
            &mut count,
            &mut get_default,
        )
    };
    (result == 0).then(|| {
        format!(
            "active={} lowpri={} timer={} disk={}",
            info[0], info[1], info[2], info[3]
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kernel_answers_for_this_process() {
        let state = nap_state().expect("task_policy_get answers for the process's own task");
        assert!(state.starts_with("active="), "{state}");
    }

    #[test]
    fn an_activity_begins_and_ends() {
        let activity = SummonActivity::begin().expect("Foundation gives an activity");
        drop(activity);
        // And again: one at a time, never kept.
        assert!(SummonActivity::begin().is_some());
    }
}
