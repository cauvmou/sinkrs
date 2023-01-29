use std::{collections::HashMap, time::Instant, sync::{Arc, Mutex}};

use crate::dns::{record::{Record, RecordData}, Question};

struct Entry {
    class: u16,
    ttl: Instant,
    data: RecordData,
}

pub struct Cache {
    map: HashMap<Question, Vec<Entry>>
}

impl Cache {
    pub fn new() -> Arc<Mutex<Self>> {
        todo!()
    }

    pub async fn get(question: &Question) -> Vec<Record> {
        todo!()
    }
}