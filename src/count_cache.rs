use std::collections::HashMap;
use std::time::{Duration, Instant};


struct CacheData {
    count: u64,
    time: Instant,
}

impl CacheData {
    fn new(cnt: u64) -> Self {
        CacheData {
            count: cnt,
            time: Instant::now(),
        }
    }
    
    fn update(&mut self, cnt: u64) {
        self.count = cnt;
        self.time = Instant::now();
    }
    
    fn is_old(&self) -> bool {
        (Instant::now() - self.time) > Duration::new(30, 0)
    }
}


pub struct CountCache {
    cache: HashMap<String, HashMap<String, CacheData>>,
    max_cache_len: usize,
    max_sub_cache_len: usize,
}

impl CountCache {
    pub fn new() -> Self {
        CountCache {
            cache: HashMap::new(),
            max_cache_len: 64,
            max_sub_cache_len: 2048,
        }
    }
    
    pub fn update(&mut self, ids: &String, page_id: &String, cnt: u64) {
        // 캐시가 비정상적으로 커질 가능성 배제.
        if self.cache.len() >= self.max_cache_len {
            self.cache.clear();
        }
        
        let max_sub_cache_len = self.max_sub_cache_len;
        self.cache.entry(ids.clone())
            .and_modify(|sub_cache| {
                // 캐시가 비정상적으로 커질 가능성 배제.
                if sub_cache.len() >= max_sub_cache_len {
                    sub_cache.clear();
                }
            })
            .or_insert(HashMap::new())
            // sub_cache
            .entry(page_id.clone())
            .and_modify(|data| data.update(cnt))
            .or_insert(CacheData::new(cnt));
    }
    
    pub fn get(&self, ids: &String, page_id: &String, check_time: bool) -> Option<u64> {
        if let Some(sub_cache) = self.cache.get(ids) {
            if let Some(data) = sub_cache.get(page_id) {
                if !check_time || !data.is_old() {
                    return Some(data.count);
                }
            }
        }
        
        return None;
    }
}
