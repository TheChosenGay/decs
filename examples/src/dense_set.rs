use std::iter::{IntoIterator, Iterator};
use std::any::Any;

pub(crate) struct DenseSet<T> {
    sparse: Vec<Option<u32>>,   // u32的实体已经非常多了
    dense_sparse_map_arr: Vec<u32>,
    dense: Vec<T>,
}

impl<T> DenseSet<T> {
    pub fn new() -> Self {
        Self { sparse: Vec::new(), dense_sparse_map_arr: Vec::new(), dense: Vec::new() }
    }
    
    pub fn add(&mut self, entity: u32, component: T) {
        if self.has_component(entity) {
            self.replace_component(entity, component);
            return;
        }
        self.dense.push(component);
        self.dense_sparse_map_arr.push(entity);
        self.add_to_sparce(entity, (self.dense_sparse_map_arr.len() - 1) as u32);
    }
    
    // 如果存在，删除返回true， 如果不存在返回false
    pub fn remove(&mut self, entity: u32) -> bool {
        if self.sparse.len() <= entity as usize || self.sparse[entity as usize].is_none() {
            return false;
        }
        
        if let Some(value) = self.sparse[entity as usize] {
            if let Some(need_change_entity) = self.remove_from_dense(value as usize) {
                self.sparse[need_change_entity as usize] = self.sparse[entity as usize];
                self.sparse[entity as usize] = None;
                return true;
            }
            return false;
        }
        
        return false;
    }
    
    fn has_component(&self, entity:u32) -> bool {
        if self.sparse.len() <= entity as usize || self.sparse[entity as usize].is_none() {
            return false;
        }
        true
    }

    fn replace_component(&mut self, entity: u32, component: T) {
        if let Some(idx) = self.sparse[entity as usize] {
            if idx >= self.dense.len() as u32 {
                return;
            }
            self.dense[idx as usize] = component;
        }
    }
    
    fn remove_from_dense(&mut self, index: usize) ->  Option<u32>{
        if self.dense.len() <= index as usize {
            return None;
        }
        
        let last_entity = self.dense_sparse_map_arr.last().unwrap().clone();
        self.dense.swap_remove(index);
        self.dense_sparse_map_arr.swap_remove(index);
        Some(last_entity)
    }
    
    fn add_to_sparce(&mut self, entity: u32, value: u32) {
        if self.sparse.len() <= entity as usize {
            self.sparse.extend(vec![None; entity as usize - self.sparse.len() + 1]);
        }
        self.sparse[entity as usize] = Some(value);
    }
}
impl<'a, T> IntoIterator for &'a mut DenseSet<T> { type Item = (u32, &'a mut T);
    type IntoIter = std::iter::Zip<
        std::iter::Copied<std::slice::Iter<'a, u32>>,
        std::slice::IterMut<'a, T>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.dense_sparse_map_arr
            .iter()
            .copied()
            .zip(self.dense.iter_mut())
    }
    
}

impl<'a, T> IntoIterator for &'a DenseSet<T> {
    type Item = (u32, &'a T);
    type IntoIter = std::iter::Zip<
        std::iter::Copied<std::slice::Iter<'a, u32>>,
        std::slice::Iter<'a, T>,
    >;
    
    fn into_iter(self) -> Self::IntoIter {
        self.dense_sparse_map_arr
            .iter()
            .copied()
            .zip(self.dense.iter())
    }
}


// 进行类型擦除
pub(crate) trait ErasedDenseSet {
    fn remove(&mut self, id: u32) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}


impl<T: 'static> ErasedDenseSet for DenseSet<T> {
    fn remove(&mut self, entity: u32) -> bool {
        self.remove(entity)
    }
fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}