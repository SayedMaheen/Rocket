#[macro_use] extern crate rocket;

use rocket::Outcome::*;
use rocket::{Request, Data};
use rocket::request::{self, FromRequest};

struct HasContentType;

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for HasContentType {
    type Error = ();

    async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        if request.content_type().is_some() {
            Success(HasContentType)
        } else {
            Forward(())
        }
    }
}

use rocket::data::{self, FromDataSimple};

impl FromDataSimple for HasContentType {
    type Error = ();

    fn from_data(request: &Request<'_>, data: Data) -> data::FromDataFuture<'static, Self, Self::Error> {
        Box::pin(futures::future::ready(if request.content_type().is_some() {
            Success(HasContentType)
        } else {
            Forward(data)
        }))
    }
}

#[post("/")]
fn rg_ct(ct: Option<HasContentType>) -> &'static str {
    ct.map_or("Absent", |_| "Present")
}

#[post("/data", data = "<_ct>", rank = 1)]
fn data_has_ct(_ct: HasContentType) -> &'static str {
    "Data Present"
}

#[post("/data", rank = 2)]
fn data_no_ct() -> &'static str {
    "Data Absent"
}

mod local_request_content_type_tests {
    use super::*;

    use rocket::Rocket;
    use rocket::local::blocking::Client;
    use rocket::http::ContentType;

    fn rocket() -> Rocket {
        rocket::ignite().mount("/", routes![rg_ct, data_has_ct, data_no_ct])
    }

    #[test]
    fn has_no_ct() {
        let client = Client::new(rocket()).unwrap();

        let req = client.post("/");
        assert_eq!(req.clone().dispatch().into_string(), Some("Absent".to_string()));
        assert_eq!(req.clone().dispatch().into_string(), Some("Absent".to_string()));
        assert_eq!(req.dispatch().into_string(), Some("Absent".to_string()));

        let req = client.post("/data");
        assert_eq!(req.clone().dispatch().into_string(), Some("Data Absent".to_string()));
        assert_eq!(req.clone().dispatch().into_string(), Some("Data Absent".to_string()));
        assert_eq!(req.dispatch().into_string(), Some("Data Absent".to_string()));
    }

    #[test]
    fn has_ct() {
        let client = Client::new(rocket()).unwrap();

        let req = client.post("/").header(ContentType::JSON);
        assert_eq!(req.clone().dispatch().into_string(), Some("Present".to_string()));
        assert_eq!(req.clone().dispatch().into_string(), Some("Present".to_string()));
        assert_eq!(req.dispatch().into_string(), Some("Present".to_string()));

        let req = client.post("/data").header(ContentType::JSON);
        assert_eq!(req.clone().dispatch().into_string(), Some("Data Present".to_string()));
        assert_eq!(req.clone().dispatch().into_string(), Some("Data Present".to_string()));
        assert_eq!(req.dispatch().into_string(), Some("Data Present".to_string()));
    }
}
