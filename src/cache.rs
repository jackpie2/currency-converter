use std::{collections::HashMap, env, error::Error, fs::File, io::BufReader, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::converter::Converter;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    contents: HashMap<String, HashMap<String, RateEntry>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RateEntry {
    rate: f64,
    last_updated_at: u64,
}

impl RateEntry {
    fn new(rate: f64, last_updated_at: u64) -> Self {
        RateEntry {
            rate,
            last_updated_at,
        }
    }

    fn is_valid(&self, duration: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.last_updated_at < duration
    }
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            contents: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self, Box<dyn Error>> {
        let path = env::current_dir()?;
        let path = path.join("cache.json");
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let cache: Cache = serde_json::from_reader(reader)?;
        Ok(cache)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = env::current_dir()?;
        let path = path.join("cache.json");
        let file = File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn get(&self, base: &str, target: &str, duration: u64) -> Option<Converter> {
        match self.contents.get(base) {
            Some(target_to_rate) => match target_to_rate.get(target) {
                Some(rate) => {
                    if rate.is_valid(duration) {
                        Some(Converter::new(
                            base.to_string(),
                            target.to_string(),
                            rate.rate,
                        ))
                    } else {
                        None
                    }
                }
                None => None,
            },
            None => None,
        }
    }

    pub fn set(&mut self, converter: &Converter) {
        let base = &converter.base;
        let target = &converter.target;
        let rate = &converter.rate;
        let last_updated_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        match self.contents.get_mut(base) {
            Some(target_to_rate) => {
                target_to_rate.insert(target.to_string(), RateEntry::new(*rate, last_updated_at));
            }
            None => {
                let mut target_to_rate = HashMap::new();
                target_to_rate.insert(target.to_string(), RateEntry::new(*rate, last_updated_at));
                self.contents.insert(base.to_string(), target_to_rate);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_get() {
        let mut cache = Cache::new();
        let converter = Converter::new("USD".to_string(), "EUR".to_string(), 0.85);
        cache.set(&converter);
        assert_eq!(cache.get("USD", "EUR", 60), Some(converter));
    }

    #[test]
    fn cache_set() {
        let mut cache = Cache::new();
        let converter = Converter::new("USD".to_string(), "EUR".to_string(), 0.85);
        cache.set(&converter);
        assert_eq!(cache.contents.len(), 1);
        assert_eq!(cache.contents.get("USD").unwrap().len(), 1);
        assert_eq!(
            cache.contents.get("USD").unwrap().get("EUR").unwrap().rate,
            0.85
        );
    }

    #[test]
    fn rate_entry_is_invalid() {
        let rate_entry = RateEntry::new(0.85, 0);
        assert!(!rate_entry.is_valid(60));
    }

    #[test]
    fn rate_entry_is_valid() {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let rate_entry = RateEntry::new(0.85, time);
        assert!(rate_entry.is_valid(100000));
    }
}
