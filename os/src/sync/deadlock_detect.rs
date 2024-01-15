// use crate::task::{current_task, current_process};

use alloc::vec::Vec;

// use crate::task::{ current_process};

/// Holds information about all mutex resources for deadlock detection.
///
/// Stored in the PCB (Process Control Block) during process creation, with one each for mutex and semaphore.
#[allow(unused)]
pub struct DeadLockDetector {
    /// Enables or disables deadlock detection.
    enable: bool,
    /// Current available count for each resource.
    available: Vec<usize>,
    /// Allocated count for each resource.
    allocation: Vec<Vec<usize>>,
    /// Requested count for each resource.
    need: Vec<Vec<usize>>,
}

#[allow(unused)]
impl DeadLockDetector {
    /// Creates a new `DeadLockDetector` and stores it in the PCB.
    pub fn new() -> Self {
        // info!("new: pid = {}", current_process().getpid());
        // The main thread doesn't go through the creation process, so we need to create it separately.
        let mut allocation = Vec::new();
        allocation.push(Vec::new());
        let mut need = Vec::new();
        need.push(Vec::new());

        // Create the DeadLockDetector instance.
        let ret = Self {
            enable: false,
            available: Vec::new(),
            allocation,
            need,
        };
        // info!("new: allocation = {:?}", ret.allocation);
        // info!("new: need = {:?}", ret.need);
        ret
    }

    /// Used during thread creation to modify `need` and `allocation`.
    pub fn set_tid(&mut self, tid: usize) {
        info!("set_tid: tid = {}", tid);
        while self.need.get(tid).is_none() {
            let mut v = Vec::new();
            let mut v2 = Vec::new();
            while v.len() < self.available.len() {
                v.push(0);
                v2.push(0);
            }
            self.need.push(v);
            self.allocation.push(v2);
        }
        info!("set_tid: allocation = {:?}", self.allocation);
        info!("set_tid: need = {:?}", self.need);
    }

    /// Called when creating a lock to add a new resource.
    ///
    /// For simplicity, deadlock detection is separately applied to mutexes and semaphores,
    /// without considering potential deadlocks caused by mixing mutexes, semaphores, and other synchronization primitives.
    pub fn update_available(&mut self, res_id: usize, num: usize) {
        // info!("update_avaiable: res_id = {}, num = {}", res_id, num);
        // Because we cannot delete a lock, res_id must be the newest.
        // assert_eq!(self.need.len(), res_id);
        // assert_eq!(self.available[0].len(), res_id);
        // assert_eq!(self.allocation[0].len(), res_id);

        // Set available and add a new resource type.
        self.available.push(num);

        // Modify the number of resource types in need and allocation.
        for i in 0..self.need.len() {
            self.need[i].push(0);
            self.allocation[i].push(0);
        }
    }

    /// Called when acquiring a lock.
    pub fn aquire_one(&mut self, res_id: usize, tid: usize) {
        // info!("aquire_one: res_id = {}, tid = {}", res_id, tid);
        // info!("aquire_one: available = {:?}", self.available);
        // info!("aquire_one: allocation = {:?}", self.allocation);
        // info!("aquire_one: need = {:?}", self.need);
        // If available, decrement available and increment allocation.
        if self.available[res_id] != 0 {
            self.available[res_id] -= 1;
            self.allocation[tid][res_id] += 1;
            // If not available, increment need.
        } else {
            self.need[tid][res_id] += 1;
        }
    }

    /// Called when releasing a lock.
    pub fn release_one(&mut self, res_id: usize, tid: usize) {
        // Decrement allocation and increment available.
        self.available[res_id] += 1;
        self.allocation[tid][res_id] -= 1;
    }

    /// If deadlock detection is not enabled, always returns true.
    pub fn detect_deadlock(&mut self, tid: usize, res_id: usize) -> bool {
        info!("detect_deadlock: enable = {}", self.enable);
        if self.enable {
            self._detect_deadlock(tid, res_id)
        } else {
            true
        }
    }

    /// Enables or disables deadlock detection.
    pub fn set_enable(&mut self, enable: bool) {
        info!("set_enable: para = {}", enable);
        info!("set_enable: enable = {}", self.enable);
        self.enable = enable;
        info!("set_enable: enable = {}", self.enable);
    }

    /// Deadlock detection algorithm.
    /// Parameters: Thread ID, requested resource ID.
    /// Returns: Whether the current request is allowed.
    fn _detect_deadlock(&mut self, tid: usize, res_id: usize) -> bool {

        info!("------------------_detect_deadlock: --------------------------");
        info!("now is:");
        info!("avail = {:?}", self.available);
        info!("alloc = {:?}", self.allocation);
        info!("need = {:?}", self.need);

        // temporarily need + 1 
        self.need[tid][res_id] += 1;
        info!("need = {:?}", self.need);
        info!("------------------_detect_deadlock: --------------------------");

        // create work and finish 
        let mut work = Vec::new();
        let mut finish = Vec::new();
        for i in self.available.iter() {
            work.push(*i);
        }
        for _ in 0..self.need.len() {
            finish.push(false);
        }

        // loop : try to finish at least one 
        loop {
            info!("loop: work = {:?},", work);
            info!("loop: fini = {:?},", finish);
            // At least find one = false
            let mut atleast_found_one = false;
            // Iterate over need
            for (index, one_need) in self.need.iter().enumerate() {
                // Skip those already finished
                if finish[index] {
                    continue;
                }
                let mut found = true;
                // Iterate over need[index], check if <= work
                for (i, n) in one_need.iter().enumerate() {
                    if n > &work[i] {
                        found = false;
                        break;
                    }
                }
                // If need[index] <= work
                if found {
                    atleast_found_one = true;
                    finish[index] = true;
                    let one_allocation = &self.allocation[index];
                    for (i, n) in one_allocation.iter().enumerate() {
                        work[i] += *n;
                    }
                    break;
                }
            }
            info!("one loop done, atleast_found_one = {}", atleast_found_one);
            // If none is found, break
            if !atleast_found_one {
                break;
            }
        }
        self.need[tid][res_id] -= 1;

        let ret = finish.iter().all(|x| x == &true);
        info!("ok, ret = {}", ret);

        ret 
    }
}
