use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashSet;
use redis::aio::Connection;
use redis_pool::{RedisPool, SingleRedisPool};
use redis_pool::connection::RedisPoolConnection;
use tonic::{transport::Server, Request, Response, Status};
use taggingservice::changer_server::{Changer, ChangerServer};
use taggingservice::{ChangeTagsRequest, ChangeTagsReply};
use tagging_service::utils;

pub mod taggingservice {
    tonic::include_proto!("taggingservice");
}

pub struct MyChanger {
    redis_pool: SingleRedisPool,
}
impl MyChanger {
}
#[tonic::async_trait]
impl Changer for MyChanger {
    async fn change_tags(
        &self,
        request: Request<ChangeTagsRequest>,
    ) -> Result<Response<ChangeTagsReply>, Status> {
        let change_tags_request = request.into_inner();
        println!("{:?}", change_tags_request);
        let original_tags = change_tags_request.original_tags;
        let next_tags = change_tags_request.next_state;
        let original = HashSet::from_iter(original_tags.iter().cloned());
        let next = HashSet::from_iter(next_tags.iter().cloned());
        let timestamp = SystemTime::now();
        let diffs = utils::get_diffs_from(&original, &next, timestamp.duration_since(UNIX_EPOCH).unwrap().as_millis(), change_tags_request.ticket_id);
        let mut conn : RedisPoolConnection<Connection> = self.redis_pool.aquire().await.unwrap();
        for diff in diffs {
            let data =  serde_json::to_string(&diff).unwrap();
            let _ : () = redis::pipe().zadd(diff.account_id, data, diff.timestamp_epoch_ms as u64).ignore().query_async(&mut conn).await.unwrap();
        }
        let _ : () = redis::pipe().publish("channel_1", 0).ignore().query_async(&mut conn).await.unwrap();
        let reply = ChangeTagsReply{};
        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let redis_url = "redis://0.0.0.0:6379/0";
    let client = redis::Client::open(redis_url).expect("Error while testing the connection");
    let pool = RedisPool::from(client);

    let changer = MyChanger{
        redis_pool: pool
    };

    Server::builder()
        .add_service(ChangerServer::new(changer))
        .serve(addr)
        .await?;
    Ok(())
}

