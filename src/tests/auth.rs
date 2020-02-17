#[cfg(test)]
mod tests {
    use crate::handlers::auth::LoginRequest;
    use crate::tests::helpers::tests::{assert_get, assert_post};

    const PATH: &str = "/api/v1/auth";

    #[actix_rt::test]
    async fn it_logs_a_user_in() {
        let params = LoginRequest {
            email: "satoshi@nakamotoinstitute.org".into(),
            password: "123456".into(),
        };
        let url = format!("{}/login", PATH);
        assert_post(&url, params).await;
    }

    #[actix_rt::test]
    async fn it_logs_a_user_out() {
        let url = format!("{}/logout", PATH);
        assert_get(&url).await;
    }
}
