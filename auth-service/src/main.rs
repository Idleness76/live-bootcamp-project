use auth_service::Application;

#[tokio::main]
async fn main() {
    let user_store = todo!();
    let app_state = todo!();

    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
