use crate::auth::{create_jwt, hash, PrivateClaim};
use crate::database::PoolType;
use crate::errors::ApiError;
use crate::handlers::user::UserResponse;
use crate::helpers::respond_ok;
use crate::models::user::find_by_auth;
use crate::validate::validate;
use actix_identity::Identity;
use actix_web::{HttpResponse, HttpRequest, HttpMessage};
use actix_web::web::{block, Data, Json};
use log::debug;
use serde::Serialize;


#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "email must be a valid email"))]
    pub email: String,

    #[validate(length(
        min = 6,
        message = "password is required and must be at least 6 characters"
    ))]
    pub password: String,
}

/// Login a user
/// Create and remember their JWT
pub async fn login(
    request: HttpRequest,
    pool: Data<PoolType>,
    params: Json<LoginRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    validate(&params)?;
        debug!("login params:{:?}", &params);

        // Validate that the email + hashed password matches
        let hashed = hash(&params.password);
        let user = block(move || find_by_auth(&pool, &params.email, &hashed)).await??;

        // Create a JWT
        let private_claim = PrivateClaim::new(user.id, user.email.clone());
        let jwt = create_jwt(private_claim)?;

        let _ = Identity::login(&request.extensions(), jwt.to_string());
        
        Ok(Json(user))
}

/// Logout a user
/// Forget their user_id
pub async fn logout(identity: Option<Identity>) -> Result<HttpResponse, ApiError> {
    if let Some(id) = identity{
        id.logout();
    }
    respond_ok()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::helpers::tests::get_data_pool;
    use actix_identity::{Identity, IdentityMiddleware};
    use actix_service::Service;
    use actix_web::{test::{self, read_body_json}, App, web, middleware::Logger, http};
    use log::info;
    


    #[actix_web::test]
    async fn it_logs_a_user_in(){
        // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
        let response = login_user().await;
        assert!(response.is_ok());
        assert!(response.ok().is_some())
    }

    #[actix_rt::test]
    async fn it_logs_a_user_out() {
        // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
        login_user().await.unwrap();
        let _response = logout_user().await;
    }

    #[allow(deprecated)]
    async fn logout_user(){
        let app =
            test::init_service(App::new()
            .wrap(Logger::default())
            .app_data(get_data_pool())
            .wrap(IdentityMiddleware::default())
            .service(web::resource("/logout").route(web::post().to(logout))))
                .await;

        let req = test::TestRequest::post()
            .uri("/logout")
            .set_json(LoginRequest {
                email: "satoshi@nakamotoinstitute.org".into(),
                password: "123456".into(),
            })
            .to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[allow(deprecated)]
    async fn login_user() -> Result<UserResponse, ApiError> {
        let app =
            test::init_service(App::new()
            .wrap(Logger::default())
            .app_data(get_data_pool())
            .wrap(IdentityMiddleware::default())
            .service(web::resource("/login").route(web::post().to(login))))
                .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(LoginRequest {
                email: "satoshi@nakamotoinstitute.org".into(),
                password: "123456".into(),
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_object: UserResponse = read_body_json(resp).await;

        Ok(response_object)
        
    }

    /// This handler uses json extractor
    async fn _test_login_handler(request: HttpRequest,pool: Data<PoolType>,params: web::Json<LoginRequest>) -> Result<Json<UserResponse>, ApiError> {
        validate(&params)?;
        info!("login params:{:?}", &params);

        // Validate that the email + hashed password matches
        let hashed = hash(&params.password);
        let user = block(move || find_by_auth(&pool, &params.email, &hashed)).await??;

        // Create a JWT
        let private_claim = PrivateClaim::new(user.id, user.email.clone());
        let jwt = create_jwt(private_claim)?;

        let _ = Identity::login(&request.extensions(), jwt.to_string());
        
        Ok(Json(user))
        // HttpResponse::Ok().json(UserResponse{ id: uuid!("00000000-0000-0000-0000-ffff00000000"), first_name: "satoshi".to_string(), last_name: "satoshi".to_string(), email: "satoshi@nakamotoinstitute.org".to_string() })
    }
}
