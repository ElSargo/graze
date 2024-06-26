use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GenerationalKey<T> {
    index: usize,
    generation: usize,
    marker: PhantomData<T>,
}
impl<T> Clone for GenerationalKey<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            generation: self.generation,
            marker: PhantomData,
        }
    }
}
impl<T> Copy for GenerationalKey<T> {}
impl<T> Ord for GenerationalKey<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe {
            let a: u128 = std::mem::transmute(*self);
            let b: u128 = std::mem::transmute(*other);
            Ord::cmp(&a, &b)
        }
    }
}
impl<T> PartialOrd for GenerationalKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe {
            let a: u128 = std::mem::transmute(*self);
            let b: u128 = std::mem::transmute(*other);
            Some(Ord::cmp(&a, &b))
        }
    }
}
impl<T> Eq for GenerationalKey<T> {}

impl<T> PartialEq for GenerationalKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenerationalMap<T> {
    data: Vec<(usize, Option<T>)>,
    free: Vec<usize>,
}

impl<T> Default for GenerationalMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GenerationalMap<T> {
    pub fn push(&mut self, item: T) -> GenerationalKey<T> {
        if let Some(index) = self.free.pop() {
            let generation = self.data[index].0 + 1;
            self.data[index] = (generation, Some(item));
            GenerationalKey {
                index,
                generation,
                marker: PhantomData,
            }
        } else {
            self.data.push((0, Some(item)));
            GenerationalKey {
                index: self.data.len() - 1,
                generation: 0,
                marker: PhantomData,
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.free.len()
    }

    pub fn remove(&mut self, key: GenerationalKey<T>) -> Option<T> {
        self.data.get_mut(key.index).and_then(|(generation, item)| {
            (*generation == key.generation).then_some(()).and_then(|_| {
                self.free.push(key.index);
                item.take()
            })
        })
    }

    pub fn get(&self, key: GenerationalKey<T>) -> Option<&T> {
        self.data.get(key.index).and_then(|(generation, item)| {
            (*generation == key.generation)
                .then_some(())
                .and(item.as_ref())
        })
    }

    pub fn get_mut(&mut self, key: GenerationalKey<T>) -> Option<&mut T> {
        self.data.get_mut(key.index).and_then(|(generation, item)| {
            (*generation == key.generation)
                .then_some(())
                .and(item.as_mut())
        })
    }

    pub fn contains_key(&self, key: GenerationalKey<T>) -> bool {
        self.data
            .get(key.index)
            .map_or(false, |(generation, item)| {
                item.is_some() && *generation == key.generation
            })
    }

    pub fn iter(&self) -> impl Iterator<Item = (GenerationalKey<T>, &'_ T)> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(index, (generation, wraped_item))| {
                wraped_item.as_ref().map(|item| {
                    (
                        GenerationalKey {
                            index,
                            generation: *generation,
                            marker: PhantomData,
                        },
                        item,
                    )
                })
            })
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (GenerationalKey<T>, &'_ mut T)> {
        self.data
            .iter_mut()
            .enumerate()
            .filter_map(|(index, (generation, wraped_item))| {
                wraped_item.as_mut().map(|item| {
                    (
                        GenerationalKey {
                            index,
                            generation: *generation,
                            marker: PhantomData,
                        },
                        item,
                    )
                })
            })
    }

    pub fn keys(&self) -> impl Iterator<Item = GenerationalKey<T>> + '_ {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(index, (generation, wraped_item))| {
                wraped_item.as_ref().map(|_| GenerationalKey {
                    index,
                    generation: *generation,
                    marker: PhantomData,
                })
            })
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(_index, (_generation, wraped_item))| wraped_item.as_ref())
    }

    #[allow(dead_code)]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data
            .iter_mut()
            .enumerate()
            .filter_map(|(_index, (_generation, wraped_item))| wraped_item.as_mut())
    }

    #[allow(dead_code)]
    const fn new() -> Self {
        Self {
            data: vec![],
            free: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn push() {
        let mut subject = GenerationalMap::new();
        subject.push(2);
        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(0, Some(2))],
                free: vec![]
            }
        );
        subject.push(8);
        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(0, Some(2)), (0, Some(8))],
                free: vec![]
            }
        );
        subject.push(121);
        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(0, Some(2)), (0, Some(8)), (0, Some(121))],
                free: vec![]
            }
        );
    }

    #[test]
    fn remove() {
        let mut subject = GenerationalMap::new();
        let first = subject.push(2);
        assert_eq!(subject.remove(first), Some(2));
        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(0, None)],
                free: vec![0]
            }
        );
        let first = subject.push(6);
        assert_eq!(subject.data.len(), 1);
        let second = subject.push(20);
        let third = subject.push(9);

        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(1, Some(6)), (0, Some(20)), (0, Some(9))],
                free: vec![]
            }
        );
        assert_eq!(subject.remove(second), Some(20));
        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(1, Some(6)), (0, None), (0, Some(9))],
                free: vec![1]
            }
        );

        assert_eq!(subject.remove(third), Some(9));
        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(1, Some(6)), (0, None), (0, None)],
                free: vec![1, 2]
            }
        );
        assert_eq!(subject.remove(first), Some(6));
        assert_eq!(
            subject,
            GenerationalMap {
                data: vec![(1, None), (0, None), (0, None)],
                free: vec![1, 2, 0]
            }
        );
    }

    #[test]
    fn len() {
        let mut subject = GenerationalMap::new();
        assert_eq!(subject.len(), 0);
        let first = subject.push(2);
        assert_eq!(subject.len(), 1);
        subject.push(3);
        assert_eq!(subject.len(), 2);
        subject.remove(first);
        assert_eq!(subject.len(), 1);
    }

    #[test]
    fn get() {
        let mut subject = GenerationalMap::new();
        let fisrt = subject.push(120);
        assert_eq!(subject.get(fisrt), Some(&120));
        assert_eq!(subject.remove(fisrt), Some(120));
        assert_eq!(subject.get(fisrt), None);

        let fisrt = subject.push(20);
        assert_eq!(subject.get(fisrt), Some(&20));
        let second = subject.push(30);
        assert_eq!(subject.get(second), Some(&30));
        assert_eq!(subject.remove(fisrt), Some(20));
        assert_eq!(subject.get(second), Some(&30));
        assert_eq!(subject.remove(second), Some(30));
    }

    #[test]
    fn get_mut() {
        let mut subject = GenerationalMap::new();
        let fisrt = subject.push(120);
        assert_eq!(subject.get_mut(fisrt), Some(&mut 120));
        assert_eq!(subject.remove(fisrt), Some(120));
        assert_eq!(subject.get_mut(fisrt), None);

        let fisrt = subject.push(20);
        assert_eq!(subject.get_mut(fisrt), Some(&mut 20));
        let second = subject.push(30);
        assert_eq!(subject.get_mut(second), Some(&mut 30));
        assert_eq!(subject.remove(fisrt), Some(20));
        assert_eq!(subject.get_mut(second), Some(&mut 30));
        assert_eq!(subject.remove(second), Some(30));
    }

    #[test]
    fn iter() {
        let mut subject = GenerationalMap::new();
        subject.push(1);
        subject.push(2);
        subject.push(6);
        let mut iterator = subject.iter();
        assert_eq!(
            iterator.next(),
            Some((
                GenerationalKey {
                    index: 0,
                    generation: 0,
                    marker: PhantomData
                },
                &1
            ))
        );
        assert_eq!(
            iterator.next(),
            Some((
                GenerationalKey {
                    index: 1,
                    generation: 0,
                    marker: PhantomData
                },
                &2
            ))
        );
        assert_eq!(
            iterator.next(),
            Some((
                GenerationalKey {
                    index: 2,
                    generation: 0,
                    marker: PhantomData
                },
                &6
            ))
        );
    }
}
