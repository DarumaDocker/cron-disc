use anyhow::Result;
use serde::{Deserialize, Serialize};

// Structure for discussion categories
#[derive(Debug, Deserialize)]
pub struct CategoryResponse {
    pub data: Option<CategoryData>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryData {
    pub repository: CategoryRepository,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRepository {
    pub discussion_categories: CategoryConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryConnection {
    pub nodes: Vec<Category>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "emojiHTML")]
    pub emoji_html: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RepositoryIdQuery {
    pub query: String,
    pub variables: RepositoryVariables,
}

#[derive(Debug, Serialize)]
pub struct RepositoryVariables {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryResponse {
    pub data: Option<RepositoryData>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryData {
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct DiscussionInput {
    pub query: String,
    pub variables: DiscussionVariables,
}

#[derive(Debug, Serialize)]
pub struct DiscussionVariables {
    pub input: CreateDiscussionInput,
}

#[derive(Debug, Serialize)]
pub struct CreateDiscussionInput {
    #[serde(rename = "repositoryId")]
    pub repository_id: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscussionResponse {
    pub data: Option<CreateDiscussionPayload>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionPayload {
    pub discussion: Option<Discussion>,
}

#[derive(Debug, Deserialize)]
pub struct Discussion {
    pub id: String,
    pub url: String,
    pub number: i32,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    pub locations: Option<Vec<ErrorLocation>>,
    pub path: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorLocation {
    pub line: i32,
    pub column: i32,
}

// Function to get discussion categories
pub async fn get_discussion_categories(
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<Vec<Category>> {
    let client = reqwest::Client::new();

    let query = r#"
        query($owner: String!, $name: String!) {
            repository(owner: $owner, name: $name) {
                discussionCategories(first: 10) {
                    nodes {
                        id
                        name
                        description
                        emojiHTML
                    }
                }
            }
        }
    "#
    .to_string();

    let variables = serde_json::json!({
        "owner": owner,
        "name": repo
    });

    let request_body = serde_json::json!({
        "query": query,
        "variables": variables
    });

    let response = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "rust-github-discussion-client")
        .json(&request_body)
        .send()
        .await?;

    let category_response: CategoryResponse = response.json().await?;

    if let Some(errors) = category_response.errors {
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        return Err(anyhow::anyhow!("GraphQL errors: {:?}", error_messages));
    }

    let categories = category_response
        .data
        .ok_or_else(|| anyhow::anyhow!("No data in response"))?
        .repository
        .discussion_categories
        .nodes;

    Ok(categories)
}

// Function to get repository node ID
pub async fn get_repository_id(token: &str, owner: &str, name: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let query = r#"
        query($owner: String!, $name: String!) {
            repository(owner: $owner, name: $name) {
                id
            }
        }
    "#
    .to_string();

    let variables = RepositoryVariables {
        owner: owner.to_string(),
        name: name.to_string(),
    };

    let request_body = RepositoryIdQuery { query, variables };

    let response = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "rust-github-discussion-client")
        .json(&request_body)
        .send()
        .await?;

    let repo_response: RepositoryResponse = response.json().await?;

    if let Some(errors) = repo_response.errors {
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        return Err(anyhow::anyhow!("GraphQL errors: {:?}", error_messages));
    }

    let repository_id = repo_response
        .data
        .ok_or_else(|| anyhow::anyhow!("No data in response"))?
        .repository
        .id;

    Ok(repository_id)
}

// Function to create a new discussion
pub async fn create_discussion(
    token: &str,
    repository_id: &str,
    category_id: &str,
    title: &str,
    body: &str,
) -> Result<Option<Discussion>> {
    let client = reqwest::Client::new();

    let query = r#"
        mutation CreateDiscussion($input: CreateDiscussionInput!) {
            createDiscussion(input: $input) {
                discussion {
                    id
                    url
                    number
                }
            }
        }
    "#
    .to_string();

    let variables = DiscussionVariables {
        input: CreateDiscussionInput {
            repository_id: repository_id.to_string(),
            category_id: category_id.to_string(),
            title: title.to_string(),
            body: body.to_string(),
        },
    };

    let request_body = DiscussionInput { query, variables };

    let response = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "rust-github-discussion-client")
        .json(&request_body)
        .send()
        .await?;

    let discussion_response: DiscussionResponse = response.json().await?;

    if let Some(errors) = discussion_response.errors {
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        return Err(anyhow::anyhow!("GraphQL errors: {:?}", error_messages));
    }

    let discussion = discussion_response
        .data
        .ok_or_else(|| anyhow::anyhow!("No data in response"))?
        .discussion;

    Ok(discussion)
}
