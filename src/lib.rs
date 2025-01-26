use schedule_flows::{schedule_cron_job, schedule_handler};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    schedule_cron_job(
        String::from("*/2 * * * *"),
        String::from("New discussion created"),
    )
    .await;
}

#[schedule_handler]
async fn handler(body: Vec<u8>) {
    println!("Received body: {:?}", body);
}
