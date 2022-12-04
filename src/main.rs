use std::sync::Arc;

mod service;

use service::Rejection;
use service::Service;
use service::Uuid;

use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

#[cfg(test)]
mod tests;

macro_rules! handle_method {
    (GET) => {
        warp::get()
    };
    (POST) => {
        warp::post()
    };
}
macro_rules! handle_path {
    ($str:literal) => {
        warp::path($str)
    };
    ($param:ty) => {
        warp::path::param::<$param>()
    };
}
macro_rules! handle {
    ($service:ident $method:tt [$($path:tt),*] $(JSON $len:expr)?) => {
        handle_method!($method).and($service.clone())
        $(.and(handle_path!($path)))*
        .and(warp::path::end())
        $(
            .and(warp::body::content_length_limit($len)
            .and(warp::body::json()))
        )?
    };
}

macro_rules! reply_with_status {
    ($status:tt $json:expr) => {
        Box::new(warp::reply::with_status(
            warp::reply::json(&$json),
            StatusCode::$status,
        ))
    };
}
macro_rules! reply {
    ($status:tt) => {
        |result| -> Box<dyn warp::Reply> {
            match result {
                Ok(val) => reply_with_status!($status val),
                Err(rejection) => match rejection {
                    Rejection::NotFound => Box::new(StatusCode::NOT_FOUND),
                    Rejection::BadAuth => reply_with_status!(BAD_REQUEST "bad_authentication"),
                    Rejection::OffLenLimit => reply_with_status!(BAD_REQUEST "off_length_limit"),
                },
            }
        }
    };
}
macro_rules! routes {
    ($first:expr, $($route:expr,)*) => {$first$(.or($route))*}
}

fn build_server() -> BoxedFilter<(impl Reply,)> {
    let service = Arc::new(Service::new());
    let service = warp::any().map(move || service.clone());
    routes![
        handle!(service POST ["pop"] JSON 1024 * 16)
            .map(|service: Arc<Service>, pop| service.post_a_new_pop(pop))
            .map(reply!(CREATED)),
        handle!(service POST ["area"] JSON 1024 * 16)
            .map(|service: Arc<Service>, area| service.get_pops_in_an_area(area))
            .map(reply!(OK)),
        handle!(service GET ["pop", Uuid])
            .map(|service: Arc<Service>, id| service.get_specific_pop(id))
            .map(reply!(OK)),
        handle!(service POST ["in", Uuid] JSON 1024 * 16)
            .map(|service: Arc<Service>, id, pep| service.post_a_pep_in_a_pop(id, pep))
            .map(reply!(CREATED)),
        handle!(service GET ["in", Uuid, usize])
            .map(|service: Arc<Service>, id, index| service.get_specific_pep(id, index))
            .map(reply!(OK)),
        handle!(service GET ["reset"])
            .map(|service: Arc<Service>| {
                service.dev_action_clear_all();
                Ok(())
            })
            .map(reply!(OK)),
        warp::options()
            .map(warp::reply)
            .with(warp::reply::with::header(
                "Access-Control-Allow-Headers",
                "content-type"
            )),
    ]
    .with(warp::reply::with::header(
        "Access-Control-Allow-Origin",
        "*",
    ))
    // .recover(|_| async {
    //     Result::<Box<dyn warp::Reply>, std::convert::Infallible>::Ok(
    //         reply_with_status!(BAD_REQUEST "bad_request"),
    //     )
    // })
    // .with(
    //     warp::cors()
    //         .allow_any_origin()
    //         .allow_methods([Method::GET, Method::POST]),
    // )
    .with(warp::log("popmap"))
    .boxed()
}

// fn test_cors() -> BoxedFilter<(impl Reply,)> {
//     warp::any()
//         .map(warp::reply)
//         .with(warp::cors().allow_origin("http://hello.com"))
//         .with(warp::log("popmap"))
//         .boxed()
// }

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    warp::serve(build_server())
        .run(([127, 0, 0, 1], 5000))
        .await
}
