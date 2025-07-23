use crate::helpers::TestApp;

#[tokio::test]
async fn signup_route_works() {
    let app = TestApp::new().await;
    let response = app.post_signup("testuser", "password123").await;
    assert_eq!(response.status().as_u16(), 200);
}
