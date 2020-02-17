use crate::auth::{decode_jwt, PrivateClaim};
use crate::models::user::AuthUser;
use actix_identity::RequestIdentity;
use actix_web::{
    dev::Payload,
    web::{HttpRequest, HttpResponse},
    Error,
    FromRequest,
};
use futures::future::{ok, err, Ready};

/// Extractor for pulling the identity out of a request.
///
/// Simply add "user: AuthUser" to a handler to invoke this.
impl FromRequest for AuthUser {
    type Error = Error;
    type Config = ();
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let identity = RequestIdentity::get_identity(req);
        if let Some(identity) = identity {
            let private_claim: PrivateClaim = decode_jwt(&identity).unwrap();
            return ok(AuthUser {
                id: private_claim.user_id.to_string(),
                email: private_claim.email,
            });
        }
        err(HttpResponse::Unauthorized().into())
    }
}
