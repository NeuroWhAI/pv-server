use hyper::status::StatusCode;
use oauth2::{self, ServiceAccountAccess};
use analytics3::Analytics as GoogleAnalytics;
use regex::{self};


fn make_hyper_client() -> hyper::Client {
    hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new()))
}


pub struct Analytics {
    hub: GoogleAnalytics<hyper::Client, ServiceAccountAccess<hyper::Client>>,
}

impl Analytics {
    pub fn new(path: &str) -> Self {
        let path = String::from(path);
        let service_key = oauth2::service_account_key_from_file(&path).unwrap();
        let service_access = ServiceAccountAccess::new(service_key, make_hyper_client());
        let hub = GoogleAnalytics::new(make_hyper_client(), service_access);
        
        Analytics {
            hub: hub,
        }
    }
    
    pub fn get_pageview(&self, ids: &str, page_id: &str) -> Result<u64, String> {
        let rgx_page_id = regex::escape(page_id);
    
        // Analytics ID인 ids와 조회수를 가져올 페이지의 page_id로 데이터 요청.
        let result = self.hub.data().ga_get(ids, "2005-01-01", "today", "ga:pageviews")
            .dimensions("ga:pagepath")
            .filters(&format!("ga:pagepath=~(^\\/{id}$)|(^\\/{id}\\?)", id=rgx_page_id))
            .doit();
        
        if let Ok((res, data)) = result {
            if res.status == StatusCode::Ok {
                let rows = data.rows.unwrap_or(vec![vec!["".into(), "0".into()]]);
                
                // 조회수 합산
                let total_views = rows.iter()
                    .map(|col| col[1].parse::<u64>().unwrap_or(0))
                    .fold(0, |total, v| total + v);
                
                Ok(total_views)
            }
            else {
                Err(format!("Error! {}", res.status))
            }
        }
        else {
            Err("Error!".to_owned())
        }
    }
}

unsafe impl Send for Analytics {}
unsafe impl Sync for Analytics {}
