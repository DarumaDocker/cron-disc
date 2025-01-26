mod discussion;

use schedule_flows::{schedule_cron_job, schedule_handler};

use discussion::*;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    schedule_cron_job(
        String::from("0 * * * *"),
        String::from("New discussion created"),
    )
    .await;
}

#[schedule_handler]
async fn handler(body: Vec<u8>) {
    let token =
        std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable is required");
    let owner =
        std::env::var("GITHUB_OWNER").expect("GITHUB_OWNER environment variable is required");
    let repo = std::env::var("GITHUB_REPO").expect("GITHUB_REPO environment variable is required");
    let category = std::env::var("DISCUSSION_CATEGORY")
        .expect("DISCUSSION_CATEGORY environment variable is required");

    let repository_id = get_repository_id(&token, &owner, &repo).await.unwrap();

    // Get available discussion categories
    let categories = get_discussion_categories(&token, &owner, &repo)
        .await
        .unwrap();

    // For this example, we'll use the first category if available
    let category_id = categories
        .iter()
        .filter_map(|c| match c.name == "General" {
            true => Some(c),
            false => None,
        })
        .next()
        .ok_or_else(|| anyhow::anyhow!("No discussion categories available"))
        .unwrap()
        .id
        .clone();

    let title = "Discussion Title";
    let body = "Discussion content goes here";

    match create_discussion(&token, &repository_id, &category_id, title, body).await {
        Ok(Some(discussion)) => {
            println!("Discussion created successfully!");
            println!("ID: {}", discussion.id);
            println!("URL: {}", discussion.url);
            println!("Number: {}", discussion.number);
        }
        Ok(None) => {
            println!("Discussion created without response");
        }
        Err(e) => {
            eprintln!("Error creating discussion: {}", e);
        }
    }
}
