use super::*;

use popmap::{GetPop, UserInfo};
use service::{Area, Location, PostPop};
use uuid::Uuid;

use crate::build_server;

// I LOVE MACRO !

// helper macro for the tester! macro
macro_rules! tester_status {
    (OK) => {
        200
    };
    (CREATED) => {
        201
    };
    (NOT_FOUND) => {
        404
    };
}

macro_rules! if_not {
    (() $code:block) => {
        $code
    };
    (($($target:tt)+) $code:block) => {};
}

// helper macro for the tester! macro
macro_rules! tester_path {
        ($($elem:expr),*) => {{
            let mut path = String::from("/");
            let elems: Vec<String> = vec![$(format!("{}", $elem)),*];
            path.push_str(elems.join("/").as_ref());
            path
        }};
    }
// This macro test the server with a rquest and check the response
// Usage:
//     tester!(server METHOD ["path" "with" "parts"] body => STATUS response-type: expected_value)
macro_rules! tester {
        ($server:ident $method:tt [$($path:tt)*] $($body:expr)? => $status:tt $($res_type:ty $(: $cmp:expr)?)?) => {
            {
                let res = warp::test::request()
                    .method(stringify!($method))
                    .path(tester_path!($($path)*).as_ref())
                    $(.json(&$body))?
                    .reply(&$server)
                    .await;
                dbg!(res.status());
                dbg!(res.body());
                assert_eq!(res.status(), tester_status!($status));
                if_not!(($($res_type)?) {
                    assert!(res.body().is_empty());
                });
                $(
                    let json: $res_type = serde_json::from_slice(res.body()).unwrap();
                    $(assert_eq!(json, $cmp);)?
                    json
                )?
            }
        };
        // ($server:ident $method:tt [$($path:tt)*] $($body:expr)? => $status:tt) => {
        //     {
        //         let res = warp::test::request()
        //             .method(stringify!($method))
        //             .path(tester_path!($($path)*).as_ref())
        //             $(.json(&$body))?
        //             .reply(&$server)
        //             .await;
        //         dbg!(res.status());
        //         dbg!(res.body());
        //         assert_eq!(res.status(), tester_status!($status));
        //     }
        // };
    }

#[tokio::test]
async fn post_and_get_popup() {
    let server = build_server();
    let lat = (0, 0, 0).try_into().unwrap();
    let lng = (0, 0, 0).try_into().unwrap();
    let area = Area {
        lat,
        lng,
        radius: 10,
    };
    let location = Location { lat, lng };
    let post_pop = PostPop {
        title: "Hello".into(),
        description: "World!".into(),
        location,
        expire: 0,
        user: UserInfo {
            id: 0,
            first_name: "David".into(),
            last_name: "Iwanaoa".into(),
            photo_url: "".into(),
        }
        .fake_auth(),
    };
    let nil = Uuid::nil();

    // The tester! macro is used as follow:
    //     tester!(server_instance METHOD [path] body => STATUS type: value)
    // This will send a request then retreive the response and compare it with the value

    // This macro invocation test that at first, no popup should be found
    tester!(server POST ["area"] area => OK Vec<Uuid>: vec![]);

    // We publish a new popup
    let id = tester!(server POST ["pop"] post_pop => CREATED Uuid);

    // We fetch it
    tester!(server GET ["pop", id] => OK GetPop);

    // We try a non-existing id
    tester!(server GET ["pop", nil] => NOT_FOUND);

    tester!(server POST ["area"] area => OK Vec<Uuid>: vec![id]);
}
