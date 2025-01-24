use std::collections::HashMap;
use std::mem::offset_of;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemStack {
    pub item_id: u16,
    pub count: u8,
}

#[derive(Debug, PartialEq)]
pub enum InventoryError {
    InvalidSlot,
    InvalidCount,
}

#[derive(Debug)]
pub struct Inventory {
    slots: HashMap<i16, ItemStack>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            slots: HashMap::new(),
        }
    }

    pub fn add_item(&mut self, item_id: u16, mut count: u8) -> u8 {
        // First pass: fill existing stacks
        for slot_num in 0..=35 {
            if let Some(existing) = self.slots.get_mut(&slot_num) {
                if existing.item_id == item_id {
                    let available = 64 - existing.count;
                    if available > 0 {
                        let add = count.min(available);
                        existing.count += add;
                        count -= add;
                        if count == 0 {
                            return 0;
                        }
                    }
                }
            }
        }

        // Second pass: fill empty slots
        for slot_num in 0..=35 {
            if !self.slots.contains_key(&slot_num) {
                let add = count.min(64);
                self.slots.insert(slot_num, ItemStack { item_id, count: add });
                count -= add;
                if count == 0 {
                    return 0;
                }
            }
        }

        count
    }

    pub fn remove_item(&mut self, item_id: u16, mut count: u8) -> u8 {
        let mut remove_ids = vec![];

        for (id, slot) in &mut self.slots.iter_mut() {
            if slot.item_id == item_id {
                if slot.count <= count {
                    count -= slot.count;
                    remove_ids.push(*id);
                } else {
                    slot.count -= count;
                    count = 0;
                }
                if count == 0 {
                    break;
                }
            }
        }

        for id in remove_ids {
            self.slots.remove(&id);
        }

        count
    }

    pub fn get_item_count(&self, item_id: u16) -> u32 {
        self.slots
            .iter()
            .filter_map(|(_, slot)| {
                if slot.item_id == item_id {
                    Some(slot.count as u32)
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn get_slot(&self, index: i16) -> Option<ItemStack> {
        self.slots.get(&index).cloned()
    }

    pub fn set_slot(
        &mut self,
        index: i16,
        item: Option<ItemStack>,
    ) {
        // TODO: Maybe filter out invalid slots?
        match item {
            None => self.slots.remove(&index),
            Some(item) => self.slots.insert(index, item),
        };
    }

    pub fn find_item_slot(&self, item_id: u16) -> Option<i16> {
        self.slots
            .iter()
            .find(|(_, slot)| slot.item_id == item_id)
            .map(|(i, _)| *i)
    }

    pub fn is_full(&self) -> bool {
        for slot_num in 0..=35 {
            if !self.slots.contains_key(&slot_num) {
                return false;
            }
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        for slot_num in 0..=35 {
            if self.slots.contains_key(&slot_num) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_inventory_is_empty() {
        let inv = Inventory::new();
        for i in 0..256 {
            assert_eq!(inv.get_slot(i), None);
        }
    }

    #[test]
    fn add_and_remove_items() {
        let mut inv = Inventory::new();
        assert_eq!(inv.add_item(1, 64), 0);
        assert_eq!(inv.get_item_count(1), 64);
        assert_eq!(inv.add_item(1, 65), 0);
        assert_eq!(inv.remove_item(1, 60), 0);
        assert_eq!(inv.get_item_count(1), 64 + 65 - 60);
    }

    #[test]
    fn slot_management() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item_id: 1, count: 10 }));
        inv.set_slot(255, Some(ItemStack { item_id: 2, count: 20 }));

        assert_eq!(
            inv.get_slot(0),
            Some(ItemStack { item_id: 1, count: 10 })
        );
        assert_eq!(
            inv.get_slot(255),
            Some(ItemStack { item_id: 2, count: 20 })
        );
    }

    #[test]
    fn inventory_capacity() {
        let mut inv = Inventory::new();
        for i in 0..=35 {
            inv.set_slot(i, Some(ItemStack { item_id: 1, count: 64 }));
        }
        assert!(inv.is_full());
        assert_eq!(inv.add_item(1, 1), 1);
    }
}