use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenerationalKey {
    index: usize,
    generation: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenerationalMap<T> {
    data: Vec<(usize, Option<T>)>,
    free: Vec<usize>,
}

impl<T> GenerationalMap<T> {
    pub fn push(&mut self, item: T) -> GenerationalKey {
        if let Some(index) = self.free.pop() {
            let generation = self.data[index].0 + 1;
            self.data[index] = (generation, Some(item));
            GenerationalKey { index, generation }
        } else {
            self.data.push((0, Some(item)));
            GenerationalKey {
                index: self.data.len() - 1,
                generation: 0,
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.free.len()
    }

    pub fn remove(&mut self, key: GenerationalKey) -> Option<T> {
        self.data.get_mut(key.index).and_then(|(generation, item)| {
            (*generation == key.generation).then_some(()).and_then(|_| {
                self.free.push(key.index);
                item.take()
            })
        })
    }

    pub fn get(&self, key: GenerationalKey) -> Option<&T> {
        self.data.get(key.index).and_then(|(generation, item)| {
            (*generation == key.generation)
                .then_some(())
                .and(item.as_ref())
        })
    }

    pub fn get_mut(&mut self, key: GenerationalKey) -> Option<&mut T> {
        self.data.get_mut(key.index).and_then(|(generation, item)| {
            (*generation == key.generation)
                .then_some(())
                .and(item.as_mut())
        })
    }

    pub fn contains_key(&self, key: GenerationalKey) -> bool {
        self.data
            .get(key.index)
            .map_or(false, |(generation, item)| {
                item.is_some() && *generation == key.generation
            })
    }

    pub const fn iter(&self) -> GenerationalIterator<'_, T> {
        GenerationalIterator {
            host: self,
            index: 0,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = GenerationalKey> + '_ {
        GenerationalIterator {
            host: self,
            index: 0,
        }
        .map(|(key, _item)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        GenerationalIterator {
            host: self,
            index: 0,
        }
        .map(|(_key, item)| item)
    }

    #[allow(dead_code)]
    const fn new() -> Self {
        Self {
            data: vec![],
            free: vec![],
        }
    }
}

pub struct GenerationalIterator<'a, T> {
    host: &'a GenerationalMap<T>,
    index: usize,
}

impl<'a, T> Iterator for GenerationalIterator<'a, T> {
    type Item = (GenerationalKey, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            println!("{}", self.index);
            if let Some((generation, Some(item))) = match self.host.data.get(self.index) {
                None => return None,
                Some((generation, item)) => Some((generation, item.as_ref())),
            } {
                self.index += 1;
                return Some((
                    GenerationalKey {
                        generation: *generation,
                        index: self.index,
                    },
                    item,
                ));
            }
            self.index += 1;
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
}