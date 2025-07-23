use crate::helpers::TestApp;

#[tokio::test]
async fn login_route_works() {
    let app = TestApp::new().await;
    let response = app.post_login("testuser", "password123").await;
    assert_eq!(response.status().as_u16(), 200);
}
