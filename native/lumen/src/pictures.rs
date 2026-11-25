use std::collections::HashMap;

struct Held<T> {
    picture: T,
    bytes: usize,
    used: u64,
}

pub struct Pictures<T> {
    held: HashMap<String, Held<T>>,
    bytes: usize,
    budget: usize,
    frame: u64,
}

impl<T: Copy> Pictures<T> {
    pub fn new(budget: usize) -> Pictures<T> {
        Pictures { held: HashMap::new(), bytes: 0, budget, frame: 0 }
    }

    pub fn next_frame(&mut self) {
        self.frame += 1;
    }

    pub fn find(&mut self, key: &str) -> Option<T> {
        let held = self.held.get_mut(key)?;
        held.used = self.frame;
        Some(held.picture)
    }

    #[must_use = "the pictures given back are no longer held: free them"]
    pub fn keep(&mut self, key: &str, picture: T, bytes: usize) -> Vec<T> {
        let mut dropped = Vec::new();
        if let Some(old) = self.held.insert(key.to_string(), Held { picture, bytes, used: self.frame }) {
            self.bytes -= old.bytes;
            dropped.push(old.picture);
        }
        self.bytes += bytes;
        while self.bytes > self.budget {
            let Some(oldest) = self.held.iter().filter(|(_, held)| held.used < self.frame).min_by_key(|(_, held)| held.used).map(|(key, _)| key.clone()) else { break };
            dropped.extend(self.forget(&oldest));
        }
        dropped
    }

    fn forget(&mut self, key: &str) -> Option<T> {
        let held = self.held.remove(key)?;
        self.bytes -= held.bytes;
        Some(held.picture)
    }

    #[must_use = "the pictures given back are no longer held: free them"]
    pub fn let_go(&mut self, which: impl Fn(&str) -> bool) -> Vec<T> {
        let keys: Vec<String> = self.held.keys().filter(|key| which(key)).cloned().collect();
        keys.iter().filter_map(|key| self.forget(key)).collect()
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn count(&self) -> usize {
        self.held.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn over_budget_the_picture_used_longest_ago_goes_first() {
        let mut pictures = Pictures::new(100);
        assert!(pictures.keep("a", 1, 40).is_empty());
        pictures.next_frame();
        assert!(pictures.keep("b", 2, 40).is_empty());
        pictures.next_frame();
        assert_eq!(pictures.find("a"), Some(1));
        pictures.next_frame();
        assert_eq!(pictures.keep("c", 3, 40), vec![2]);
        assert_eq!(pictures.find("b"), None);
        assert_eq!(pictures.find("a"), Some(1));
        assert_eq!(pictures.bytes(), 80);
    }

    #[test]
    fn a_picture_used_in_this_frame_is_kept_even_over_budget() {
        let mut pictures = Pictures::new(50);
        assert!(pictures.keep("a", 1, 40).is_empty());
        assert!(pictures.keep("b", 2, 40).is_empty());
        assert_eq!(pictures.bytes(), 80);
        pictures.next_frame();
        assert_eq!(pictures.keep("c", 3, 10).len(), 1);
        assert_eq!(pictures.bytes(), 50);
    }

    #[test]
    fn a_picture_made_again_under_the_same_name_gives_the_old_one_back() {
        let mut pictures = Pictures::new(100);
        assert!(pictures.keep("a", 1, 40).is_empty());
        assert_eq!(pictures.keep("a", 2, 30), vec![1]);
        assert_eq!(pictures.bytes(), 30);
        assert_eq!(pictures.find("a"), Some(2));
    }

    #[test]
    fn letting_go_gives_back_only_the_pictures_asked_for() {
        let mut pictures = Pictures::new(100);
        let _ = pictures.keep("icon:a", 1, 10);
        let _ = pictures.keep("tile:a", 2, 30);
        assert_eq!(pictures.let_go(|key| !key.starts_with("icon:")), vec![2]);
        assert_eq!(pictures.bytes(), 10);
        assert_eq!(pictures.count(), 1);
    }
}
