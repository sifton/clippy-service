extern crate iron;
#[macro_use]
extern crate router;
extern crate staticfile;
extern crate redis;

use std::path::Path;
use std::env;

use redis::Commands;
use iron::prelude::*;
use staticfile::Static;
use router::Router;
use iron::status;
use iron::modifiers::Redirect;
use iron::Url;

fn main() {

    let router = router!(
        get "/github/:user/:repo/:branch/badge" => github_handler,
        get "/github/:user/:repo/badge" => github_handler,
        get "/" => Static::new(Path::new("static"))
    );

    println!("Check out 8080");
    Iron::new(router).http("0.0.0.0:8080").unwrap();

    fn github_handler(req: &mut Request) -> IronResult<Response> {

        let ref router = req.extensions.get::<Router>().unwrap();
        let user = router.find("user").unwrap();
        let repo = router.find("repo").unwrap();
        let branch = router.find("branch").unwrap_or("master");
        let key = format!("{}/{}:{} ", user, repo, branch);
        let redis = setup_redis();

        if let Some(url) = get_redis_redir(&redis, &key){
            return Ok(Response::with((Redirect(url))));
        }


        let resp = format!("There shall be content here for {}", key);
        Ok(Response::with((status::Ok, resp)))

    }
}

fn get_redis_redir(redis: &redis::Connection, key: &str) -> Option<Url> {
    let result : Option<String> = get_redis_value(redis, key);
    if result.is_some(){
        if let Ok(url) = Url::parse(&result.unwrap()){
            return Some(url);
        }
    }
    return None;
}

fn get_redis_value(redis: &redis::Connection, key: &str) -> Option<String> {

    let cached_result = redis.get(key);

    if cached_result.is_ok(){
        let cached_value : Option<String>  = cached_result.unwrap();
        if cached_value.is_some(){
            return cached_value;
        }
    }

    return None;
}

fn setup_redis() -> redis::Connection<> {
    let url = redis::parse_redis_url(
            &env::var("REDIS_URL").unwrap_or("redis://redis/".to_string())
        ).unwrap();
    redis::Client::open(url
            ).unwrap(
            ).get_connection(
            ).unwrap()
}
