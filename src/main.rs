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
use rocket::State;
use analytics::Analytics;
use count_cache::CountCache;


type LockCountCache = Arc<RwLock<CountCache>>;


#[get("/")]
fn index() -> &'static str {
    "PV-Server"
}

#[get("/pv/<ids>/<page_id>")]
fn get_pageview(ids: String, page_id: String,
    service: State<Analytics>, cache: State<LockCountCache>) -> String
{
    // 캐시에 유효한 데이터가 있으면 바로 반환.
    {
        let r_cache = cache.read().unwrap();
        
        if let Some(cnt) = r_cache.get(&ids, &page_id, true) {
            return cnt.to_string();
        }
    }

    // API 호출.
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


fn main() {
    let service = Analytics::new("key.json");
    let cache = Arc::new(RwLock::new(CountCache::new()));

    rocket::ignite()
        .manage(service)
        .manage(cache)
        .mount("/", routes![index, get_pageview])
        .launch();
}
