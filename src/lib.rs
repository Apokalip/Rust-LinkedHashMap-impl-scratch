#![allow(dead_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;
use std::borrow::Borrow;

const INITIAL_NBUCKETS: usize = 1;

pub struct HashMap<K, V> {
	buckets: Vec <Vec<(K, V)>>,
	items: usize,
}

impl<K, V> HashMap<K, V> 
where K: Hash + Eq
{
	pub fn new() -> Self {
		HashMap{
			buckets: Vec::new(),
			items: 0,
		}
	}

	//key hasher helper
	fn bucket_hash<Q>(&self, key: &Q) -> usize
	where
	K: Borrow<Q>,
	Q: Hash + Eq + ?Sized,
	{
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		// do the modulo in u64 so we dont lose data in 32bit systems, then covert to usize
		(hasher.finish() % (self.buckets.len() as u64)) as usize
	}

	pub fn insert(&mut self, key: K, value: V) -> Option<V> {
		
		// check for resize, if empty or if 3/4 full call resize
		// can check to risize or not in multiple way, does not matter in the learning case
		if self.buckets.is_empty() || (self.items > 3 * self.buckets.len() / 4) {
			self.resize();
		};

		let bucket = self.bucket_hash(&key);
		let bucket = &mut self.buckets[bucket];

		self.items += 1;

		for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
			// there is an existing key
			if ekey == &key {
				return Some(mem::replace(evalue, value));
			}
		}

		//if we get to here then there was not an existing key so we add it
		bucket.push((key,value));
		None
	}
	
	pub fn get<Q>(&self, key: &Q) -> Option<&V> 
	where 
		K: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		let bucket = self.bucket_hash(key);
		
		self.buckets[bucket]
		.iter()
		.find(|&(ref ekey, _)| ekey.borrow() == key)
		.map(|(_, ref v)| v)
	}
	
	pub fn contains_key<Q>(&self, key: &Q) -> bool 
	where 
		K: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		self.get(key).is_some()
	}

	pub fn remove<Q>(&mut self, key: &Q) -> Option<V> 
	where 
		K: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		let bucket = self.bucket_hash(key);
		let bucket = &mut self.buckets[bucket];
		let i = bucket.iter().position(|&(ref ekey, _)| ekey.borrow() == key)?;
		self.items -= 1;
		Some(bucket.swap_remove(i).1)
	}

	pub fn len(&self) -> usize {
		self.items
	}

	pub fn is_empty(&self) -> bool {
		self.items == 0
	}

	//resize and move buckets
	fn resize(&mut self) {
		let target_size = match self.buckets.len() {
			// setting low starting size so its easier to test resizes
			0 => INITIAL_NBUCKETS,
			n => 2 * n,
		};

		//create buckets 
		let mut new_buckets = Vec::with_capacity(target_size);
		//create vector inside every bucket
		new_buckets.extend((0..target_size).map(|_| Vec::new()));

		for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
			let mut hasher = DefaultHasher::new();
			key.hash(&mut hasher);
			// do the modulo in u64 so we dont lose data in 32bit systems, then covert to usize
			let bucket: usize= (hasher.finish() % (new_buckets.len() as u64)) as usize;
			new_buckets[bucket].push((key,value));
		}

		mem::replace(&mut self.buckets, new_buckets);
	}
}

pub struct Iter<'a, K, V> {
	map: &'a HashMap<K, V>,
	bucket: usize,
	at: usize,
}

// impl<'a, K, V> Iter<'a, K, V> {
// 	fn new(&'a HashMap<K,V>) -> Self {
// 	}
// }

impl<'a, K, V> Iterator for Iter<'a, K, V>{
	type Item = (&'a K, &'a V);
	fn next(&mut self) -> Option<Self::Item>{
		loop{
			match self.map.buckets.get(self.bucket){
				Some(bucket) => {
					match bucket.get(self.at){
						Some(&(ref k, ref v)) =>{
							//move along at and bucket
							self.at += 1;
							break Some((k,v))
						}
						None=> {
							//reached the end of current bucket go to next bucket start
							self.bucket += 1;
							self.at = 0;
							continue;
						}
					}
				}
				//spilled over last bucket
				None => break None,
			}
		}
	}
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
	type Item = (&'a K, &'a V);
	type IntoIter = Iter<'a, K, V>;
	fn into_iter(self) -> Self::IntoIter {
		Iter{
			map: self,
			bucket: 0,
			at: 0,
		}
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn insert() {
		let mut map = HashMap::new();
		assert_eq!(map.len(),0);
		assert!(map.is_empty());
		map.insert("foo",42);
		assert_eq!(map.len(),1);
		assert!(!map.is_empty());
		assert_eq!(map.get(&"foo"), Some(&42));
		assert_eq!(map.remove(&"foo"), Some(42));
		assert_eq!(map.get(&"foo"), None);
		assert!(map.is_empty());
	}

	#[test]
	fn iter() {
		let mut map = HashMap::new();

		map.insert("foo",42);
		map.insert("bar",43);
		map.insert("baz",142);
		map.insert("quox",7);

		for (&k, &v) in &map{
			match k{
				"foo" => assert_eq!(v, 42),
				"bar" => assert_eq!(v, 43),
				"baz" => assert_eq!(v, 142),
				"quox" => assert_eq!(v, 7),
				_=> unreachable!()
			}
		}
	}
}