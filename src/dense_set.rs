use std::iter::{IntoIterator, Iterator};
use std::any::Any;
use crate::entity::Entity;

pub(crate) struct DenseItem<T> {
    pub id: u64,
    pub item: T,
} 

impl<T> DenseItem<T> {
   pub fn new(id: u64, item: T) -> Self {
    Self { id, item }
   }

}


pub(crate) struct DenseSet<T> {
    dense: Vec<DenseItem<T>>,
}

impl<T> DenseSet<T> {
    pub fn new() -> Self {
        Self { dense: Vec::new() }
    }
    
    pub fn add(&mut self, entity: Entity, component: T) {
        self.dense.push(DenseItem::<T>::new(entity.id as u64, component));
    }
}

impl<'a, T> IntoIterator for &'a mut DenseSet<T> {
    type Item = &'a mut DenseItem<T>;
    type IntoIter = std::slice::IterMut<'a, DenseItem<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.dense.iter_mut()
    }
    
}

impl<'a, T> IntoIterator for &'a DenseSet<T> {
    type Item = &'a DenseItem<T>;
    type IntoIter = std::slice::Iter<'a, DenseItem<T>>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.dense.iter()
    }
}


// 进行类型擦除
pub(crate) trait ErasedDenseSet {
    fn remove(&mut self, id: u64) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}


impl<T: 'static> ErasedDenseSet for DenseSet<T> {
    fn remove(&mut self, id: u64) -> bool {
        self.dense.iter().position(|item| item.id == id).map(|index| {
            self.dense.remove(index);
            true
        }).unwrap_or(false)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}