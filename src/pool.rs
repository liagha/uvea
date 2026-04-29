use crate::types::{Identifier, PoolEntry};

pub struct Pool<T, const N: usize> {
    pub entries: [PoolEntry; N],
    pub items: [T; N],
}

impl<T: Default + Copy, const N: usize> Pool<T, N> {
    pub fn new(default_item: T) -> Self {
        Self {
            entries: [PoolEntry::default(); N],
            items: [default_item; N],
        }
    }

    pub fn get_index(&self, identifier: Identifier) -> Option<usize> {
        self.entries.iter().position(|e| e.identifier == identifier)
    }

    pub fn get(&self, identifier: Identifier) -> Option<&T> {
        self.get_index(identifier).map(|idx| &self.items[idx])
    }

    pub fn get_or_insert(&mut self, identifier: Identifier, frame: u32) -> (usize, &mut T) {
        if let Some(idx) = self.get_index(identifier) {
            self.entries[idx].update = frame;
            return (idx, &mut self.items[idx]);
        }
        let slot = self
            .entries
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.update)
            .expect("pool capacity must be > 0")
            .0;

        self.entries[slot] = PoolEntry {
            identifier,
            update: frame,
        };
        (slot, &mut self.items[slot])
    }

    pub fn update(&mut self, identifier: Identifier, frame: u32) -> bool {
        if let Some(idx) = self.get_index(identifier) {
            self.entries[idx].update = frame;
            true
        } else {
            false
        }
    }
}