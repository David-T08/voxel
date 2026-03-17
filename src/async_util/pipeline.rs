use std::{
    collections::{HashSet, VecDeque},
    fmt::{Debug, Display},
    hash::Hash,
};

#[derive(Default, Debug)]
pub struct PipelineQueue<K> {
    pub to_run: VecDeque<K>,
    pub queued: HashSet<K>,
    pub running: HashSet<K>,
}

pub struct VersionedTask<K: Debug, T> {
    pub key: K,
    pub version: u32,
    pub data: T,
}

impl<K: Debug, T> Display for VersionedTask<K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Task(key={:?}, v={})]", self.key, self.version)
    }
}

impl<K> Display for PipelineQueue<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PipelineQueue {{ to_run: {}, queued: {}, running: {} }}",
            self.to_run.len(),
            self.queued.len(),
            self.running.len()
        )
    }
}

impl<K: Copy + Eq + PartialEq + Hash> PipelineQueue<K> {
    pub fn enqueue_back(&mut self, key: K) -> bool {
        if self.running.contains(&key) {
            return false;
        }

        if self.queued.insert(key) {
            self.to_run.push_back(key);
            return true;
        }

        false
    }

    pub fn enqueue_front(&mut self, key: K) -> bool {
        if self.running.contains(&key) {
            return false;
        }

        if self.queued.insert(key) {
            self.to_run.push_front(key);
            return true;
        }

        false
    }

    pub fn pop_next(&mut self) -> Option<K> {
        let key = self.to_run.pop_front()?;
        self.queued.remove(&key);
        self.running.insert(key);
        Some(key)
    }

    pub fn mark_running(&mut self, key: K) {
        self.running.insert(key);
    }

    pub fn finish(&mut self, key: &K) {
        self.running.remove(key);
    }

    pub fn is_busy(&self, key: &K) -> bool {
        self.queued.contains(key) || self.running.contains(key)
    }

    pub fn cancel(&mut self, key: &K) {
        self.queued.remove(key);
        self.running.remove(key);
        self.to_run.retain(|k| k != key);
    }
}
