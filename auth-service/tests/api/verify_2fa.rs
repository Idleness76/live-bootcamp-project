use crate::helpers::TestApp;

#[tokio::test]
async fn verify_2fa_route_works() {
    let app = TestApp::new().await;
    let response = app.post_verify_2fa("123456").await;
    assert_eq!(response.status().as_u16(), 200);
}
