use actix_web::{App, get, HttpRequest, HttpResponse, HttpServer, post, Responder, web, middleware::Logger};
use actix_web::body::BoxBody;
use actix_web::web::Redirect;
use serde::{Deserialize, Serialize};
use redis;
use std::env;
use env_logger;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use redis::{Commands, RedisResult};


#[derive(Clone)]
struct AppConfig {
    shorter_url: String
}

impl AppConfig{
    fn new(shorter_url: String) -> Self {
        Self{shorter_url}
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_dsn = env::var("REDIS_DSN").expect("REDIS_DSN not found");
    let shorter_url = env::var("SHORTER_URL").expect("SHORTER_URL not found");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(
        move| |{
            App::new()
                .wrap(Logger::new("%a %r %s %b %{Referer}i %{User-Agent}i %T"))
                .app_data(web::Data::new(AppConfig::new(shorter_url.clone())))
                .app_data(web::Data::new(redis::Client::open(redis_dsn.clone()).expect("Can't create redis connection")))
                .service(handle_link)
                .service(create_link)
                .service(not_found)
        }
    )
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}




#[derive(Serialize, Deserialize, Clone)]
struct Link{
    url: String
}

impl Link {
    fn new(url: String) -> Self {
        Self{url}
    }
}

impl Responder for Link{
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().json(&self)
    }
}


#[post("/create-link")]
async fn create_link(
    new_link: actix_web::web::Json<Link>,
    cache: web::Data<redis::Client>,
    app_config: web::Data<AppConfig>
) -> Link {
    let mut conn = cache.get_connection().unwrap();
    let link = new_link.into_inner().url;
    let short_code = generate_random_string();
    let service_url = app_config.into_inner().shorter_url.clone();
    conn.set::<String,String,String>(short_code.clone(), link).expect("TODO: panic message");
    Link::new(format!("{}/link/{}", service_url, &short_code))
}

#[get("/link/{short_code}")]
async fn handle_link(
    cache: web::Data<redis::Client>,
    short_code: web::Path<String>
) -> Redirect {
    let mut conn = cache.get_connection().unwrap();

    let link :RedisResult<String> = conn.get(short_code.into_inner().clone());

    match link {
        Ok(value) => Redirect::to(value).temporary(),
        Err(_e) => Redirect::to("/not-found").temporary()
    }

}

#[get("/not-found")]
async fn not_found(
) -> impl Responder {
    HttpResponse::NotFound().body("not found")
}

fn generate_random_string() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect()
}





