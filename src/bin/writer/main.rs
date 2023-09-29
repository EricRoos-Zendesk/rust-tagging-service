use redis::Commands;
use std::collections::{HashMap, HashSet};
use tagging_service::utils::{TaggingDelta, TaggingOperation};

trait TagStorable {
    fn commit_to_store(self, store: &mut TagStore);
}

#[derive(Default)]
struct TagStore {
    data: HashMap<i64, HashSet<String>>
}


impl TagStorable for TaggingDelta {
    fn commit_to_store(self, store: &mut TagStore) {
        let ticket_tag_set = store.data.get_mut(&self.ticket_id);
        match ticket_tag_set {
            Some(set) => {
                match self.operation {
                    TaggingOperation::Add(val) => {
                        set.insert(val);
                    },
                    TaggingOperation::Remove(val) => {
                        set.remove(&val);
                    }
                }
            },
            None => {
                store.data.insert(self.ticket_id, HashSet::new());
                self.commit_to_store(store);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut pubsubcon = client.get_connection()?;
    let mut pubsub = pubsubcon.as_pubsub();
    pubsub.subscribe("channel_1")?;
    let mut tag_store = TagStore::default();
    let mut con = client.get_connection()?;
    loop {
        pubsub.get_message()?;
        let size :i64 = con.zcard(0).unwrap();
        match con.zpopmin(0, size as isize).unwrap() {
            redis::Value::Bulk(bulk_data) => {
                for value in bulk_data {
                    match value {
                        redis::Value::Data(data) => {
                            let diff : Result<TaggingDelta, _> = serde_json::from_slice(&data as &[u8]);
                            match diff {
                                Ok(delta) => {
                                    delta.commit_to_store(&mut tag_store);
                                }
                                Err(_) => {}
                            }
                        },
                        _ => {}
                    }
                }
            },
            missing => {
                println!("Got something, but not sure what, {:?}", missing);
            }
        }

        println!("New State: {:?}", tag_store.data);
    }
}
