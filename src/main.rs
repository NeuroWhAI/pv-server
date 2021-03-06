#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_analytics3 as analytics3;
extern crate regex;

mod analytics;
mod count_cache;

use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use rocket::State;
use rocket::fairing::AdHoc;
use rocket::response::NamedFile;
use analytics::{Analytics, AnalyticsKey};
use count_cache::CountCache;


type LockCountCache = Arc<RwLock<CountCache>>;


#[get("/")]
fn index() -> &'static str {
    "PV-Server"
}

#[get("/pv/<ids>/<page_id>")]
fn get_pageview(ids: String, page_id: String,
    key: State<AnalyticsKey>, cache: State<LockCountCache>) -> String
{
    // 캐시에 유효한 데이터가 있으면 바로 반환.
    {
        let r_cache = cache.read().unwrap();
        
        if let Some(cnt) = r_cache.get(&ids, &page_id, true) {
            return cnt.to_string();
        }
    }

    // API 호출.
    let service = Analytics::new(&key);
    let result = service.get_pageview(&ids, &page_id);

    match result {
        Ok(data) => {
            // 캐시 업데이트.
            let mut w_cache = cache.write().unwrap();
            w_cache.update(&ids, &page_id, data);

            data.to_string()
        },
        Err(err) => {
            // 에러시 캐시의 데이터가 비록 오래 되었더라도
            // 존재만 한다면 그 데이터를 반환.
            {
                let r_cache = cache.read().unwrap();

                if let Some(cnt) = r_cache.get(&ids, &page_id, false) {
                    return cnt.to_string();
                }
            }
            
            // 실패할 API 요청을 계속 보내는 것을 방지하기 위해
            // 임의 데이터를 캐시에 기록.
            let mut w_cache = cache.write().unwrap();
            w_cache.update(&ids, &page_id, 0);

            err
        }
    }
}

#[get("/robots.txt")]
fn get_robots() -> &'static str {
    "User-agent: *\nDisallow: /"
}

#[get("/favicon.ico")]
fn get_favicon() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/favicon.ico")).ok()
}

#[get("/static/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}


fn main() {
    let key = AnalyticsKey::new("key.json");
    let cache = Arc::new(RwLock::new(CountCache::new()));

    rocket::ignite()
        .attach(AdHoc::on_response("CORS", |_, rsp| {
            rsp.set_raw_header("Access-Control-Allow-Origin", "*");
            rsp.set_raw_header("Access-Control-Allow-Methods", "GET");
            rsp.set_raw_header("Access-Control-Max-Age", "3600");
            rsp.set_raw_header("Access-Control-Allow-Headers", "Origin,Accept,X-Requested-With,Content-Type,Access-Control-Request-Method,Access-Control-Request-Headers,Authorization");
        }))
        .manage(key)
        .manage(cache)
        .mount("/", routes![index, get_pageview])
        .mount("/", routes![get_robots, get_favicon, files])
        .launch();
}
