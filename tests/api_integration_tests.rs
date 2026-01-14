//! Integration tests for TickTick API client using mock server

use wiremock::matchers::{bearer_token, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use tickrs::api::{
    CreateProjectRequest, CreateTaskRequest, TickTickClient, UpdateProjectRequest,
    UpdateTaskRequest,
};
use tickrs::models::{ChecklistItemRequest, Priority};

/// Helper to create a test client pointing at mock server
fn test_client(server: &MockServer) -> TickTickClient {
    TickTickClient::with_token_and_base_url("test_token".to_string(), server.uri())
        .expect("Failed to create test client")
}

// =============================================================================
// Project API Tests
// =============================================================================

#[tokio::test]
async fn test_list_projects_success() {
    let mock_server = MockServer::start().await;

    // Mock response with one project (INBOX will be added by client)
    let response_body = r##"[
        {
            "id": "proj123",
            "name": "Work",
            "color": "#FF5733",
            "sortOrder": 0,
            "closed": false,
            "viewMode": "list",
            "kind": "TASK"
        }
    ]"##;

    Mock::given(method("GET"))
        .and(path("/project"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let projects = client.list_projects().await.unwrap();

    // Should have INBOX + 1 project from API
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].id, "inbox"); // INBOX is added first
    assert_eq!(projects[1].id, "proj123");
    assert_eq!(projects[1].name, "Work");
}

#[tokio::test]
async fn test_get_project_success() {
    let mock_server = MockServer::start().await;

    let response_body = r##"{
        "id": "proj123",
        "name": "Work",
        "color": "#FF5733",
        "sortOrder": 0,
        "closed": false,
        "viewMode": "list",
        "kind": "TASK"
    }"##;

    Mock::given(method("GET"))
        .and(path("/project/proj123"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let project = client.get_project("proj123").await.unwrap();

    assert_eq!(project.id, "proj123");
    assert_eq!(project.name, "Work");
    assert_eq!(project.color, "#FF5733");
}

#[tokio::test]
async fn test_get_project_inbox_special_case() {
    let mock_server = MockServer::start().await;

    // No mock needed - INBOX is handled locally
    let client = test_client(&mock_server);
    let project = client.get_project("inbox").await.unwrap();

    assert_eq!(project.id, "inbox");
    assert_eq!(project.name, "Inbox");
}

#[tokio::test]
async fn test_get_project_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/project/nonexistent"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.get_project("nonexistent").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_create_project_success() {
    let mock_server = MockServer::start().await;

    let response_body = r##"{
        "id": "new_proj",
        "name": "New Project",
        "color": "#00AAFF",
        "sortOrder": 1,
        "closed": false,
        "viewMode": "list",
        "kind": "TASK"
    }"##;

    Mock::given(method("POST"))
        .and(path("/project"))
        .and(bearer_token("test_token"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(201).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let request = CreateProjectRequest {
        name: "New Project".to_string(),
        color: Some("#00AAFF".to_string()),
        view_mode: Some("list".to_string()),
        kind: None,
    };

    let project = client.create_project(&request).await.unwrap();

    assert_eq!(project.id, "new_proj");
    assert_eq!(project.name, "New Project");
}

#[tokio::test]
async fn test_update_project_success() {
    let mock_server = MockServer::start().await;

    let response_body = r##"{
        "id": "proj123",
        "name": "Updated Name",
        "color": "#FF5733",
        "sortOrder": 0,
        "closed": false,
        "viewMode": "list",
        "kind": "TASK"
    }"##;

    Mock::given(method("POST"))
        .and(path("/project/proj123"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let request = UpdateProjectRequest {
        name: Some("Updated Name".to_string()),
        color: None,
        closed: None,
        view_mode: None,
    };

    let project = client.update_project("proj123", &request).await.unwrap();

    assert_eq!(project.name, "Updated Name");
}

#[tokio::test]
async fn test_update_inbox_project_fails() {
    let mock_server = MockServer::start().await;

    let client = test_client(&mock_server);
    let request = UpdateProjectRequest {
        name: Some("Cannot Update".to_string()),
        color: None,
        closed: None,
        view_mode: None,
    };

    let result = client.update_project("inbox", &request).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("INBOX"));
}

#[tokio::test]
async fn test_delete_project_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/project/proj123"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.delete_project("proj123").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_inbox_project_fails() {
    let mock_server = MockServer::start().await;

    let client = test_client(&mock_server);
    let result = client.delete_project("inbox").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("INBOX"));
}

// =============================================================================
// Task API Tests
// =============================================================================

#[tokio::test]
async fn test_list_tasks_success() {
    let mock_server = MockServer::start().await;

    let response_body = r##"{
        "project": {
            "id": "proj123",
            "name": "Work",
            "color": "#FF5733",
            "sortOrder": 0,
            "closed": false,
            "viewMode": "list",
            "kind": "TASK"
        },
        "tasks": [
            {
                "id": "task1",
                "projectId": "proj123",
                "title": "Task One",
                "content": "",
                "isAllDay": false,
                "priority": 0,
                "status": 0,
                "sortOrder": 0,
                "timeZone": "UTC",
                "items": [],
                "reminders": [],
                "tags": []
            },
            {
                "id": "task2",
                "projectId": "proj123",
                "title": "Task Two",
                "content": "Description",
                "isAllDay": true,
                "priority": 3,
                "status": 0,
                "sortOrder": 1,
                "timeZone": "UTC",
                "items": [],
                "reminders": [],
                "tags": ["work"]
            }
        ],
        "columns": []
    }"##;

    Mock::given(method("GET"))
        .and(path("/project/proj123/data"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let tasks = client.list_tasks("proj123").await.unwrap();

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "task1");
    assert_eq!(tasks[0].title, "Task One");
    assert_eq!(tasks[1].id, "task2");
    assert_eq!(tasks[1].priority, Priority::Medium);
}

#[tokio::test]
async fn test_get_task_success() {
    let mock_server = MockServer::start().await;

    let response_body = r##"{
        "id": "task123",
        "projectId": "proj123",
        "title": "My Task",
        "content": "Task description",
        "isAllDay": false,
        "priority": 5,
        "status": 0,
        "sortOrder": 0,
        "timeZone": "America/New_York",
        "items": [],
        "reminders": [],
        "tags": ["important"]
    }"##;

    Mock::given(method("GET"))
        .and(path("/project/proj123/task/task123"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let task = client.get_task("proj123", "task123").await.unwrap();

    assert_eq!(task.id, "task123");
    assert_eq!(task.title, "My Task");
    assert_eq!(task.priority, Priority::High);
    assert_eq!(task.tags, vec!["important"]);
}

#[tokio::test]
async fn test_create_task_success() {
    let mock_server = MockServer::start().await;

    let response_body = r##"{
        "id": "new_task",
        "projectId": "proj123",
        "title": "New Task",
        "content": "",
        "isAllDay": false,
        "priority": 1,
        "status": 0,
        "sortOrder": 0,
        "timeZone": "UTC",
        "items": [],
        "reminders": [],
        "tags": []
    }"##;

    Mock::given(method("POST"))
        .and(path("/task"))
        .and(bearer_token("test_token"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(201).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let request = CreateTaskRequest {
        title: "New Task".to_string(),
        project_id: "proj123".to_string(),
        content: None,
        is_all_day: None,
        start_date: None,
        due_date: None,
        priority: Some(1),
        time_zone: None,
        tags: None,
        items: None,
    };

    let task = client.create_task(&request).await.unwrap();

    assert_eq!(task.id, "new_task");
    assert_eq!(task.title, "New Task");
    assert_eq!(task.priority, Priority::Low);
}

#[tokio::test]
async fn test_delete_task_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/project/proj123/task/task123"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.delete_task("proj123", "task123").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_complete_task_success() {
    let mock_server = MockServer::start().await;

    // Complete endpoint returns empty JSON object
    Mock::given(method("POST"))
        .and(path("/project/proj123/task/task123/complete"))
        .and(bearer_token("test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.complete_task("proj123", "task123").await;

    assert!(result.is_ok());
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_unauthorized_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/project"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.list_projects().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("expired") || err.to_string().contains("Invalid"));
}

#[tokio::test]
async fn test_rate_limit_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/project"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.list_projects().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Rate limited"));
}

#[tokio::test]
async fn test_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/project"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.list_projects().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Server error"));
}

#[tokio::test]
async fn test_bad_request_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/project"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Invalid name format"))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let request = CreateProjectRequest {
        name: "".to_string(),
        color: None,
        view_mode: None,
        kind: None,
    };

    let result = client.create_project(&request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Bad request"));
}

#[tokio::test]
async fn test_invalid_json_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/project"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.list_projects().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("parse"));
}

// =============================================================================
// Subtask (Checklist Items) Tests
// =============================================================================

#[tokio::test]
async fn test_create_task_with_subtasks() {
    let mock_server = MockServer::start().await;

    // Response includes the created task with subtasks
    let response_body = r##"{
        "id": "task_with_subs",
        "projectId": "proj123",
        "title": "Task with subtasks",
        "isAllDay": false,
        "content": "",
        "priority": 0,
        "status": 0,
        "tags": [],
        "items": [
            {"id": "sub1", "title": "Subtask 1", "status": 0, "completedTime": 0, "isAllDay": false, "sortOrder": 0, "timeZone": ""},
            {"id": "sub2", "title": "Subtask 2", "status": 0, "completedTime": 0, "isAllDay": false, "sortOrder": 1, "timeZone": ""}
        ],
        "reminders": [],
        "sortOrder": 0,
        "timeZone": ""
    }"##;

    Mock::given(method("POST"))
        .and(path("/task"))
        .and(bearer_token("test_token"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(201).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let request = CreateTaskRequest {
        title: "Task with subtasks".to_string(),
        project_id: "proj123".to_string(),
        content: None,
        is_all_day: None,
        start_date: None,
        due_date: None,
        priority: None,
        time_zone: None,
        tags: None,
        items: Some(vec![
            ChecklistItemRequest::new("Subtask 1"),
            ChecklistItemRequest::new("Subtask 2").with_sort_order(1),
        ]),
    };

    let task = client.create_task(&request).await.unwrap();

    assert_eq!(task.id, "task_with_subs");
    assert_eq!(task.title, "Task with subtasks");
    assert_eq!(task.items.len(), 2);
    assert_eq!(task.items[0].title, "Subtask 1");
    assert_eq!(task.items[1].title, "Subtask 2");
}

#[tokio::test]
async fn test_update_task_add_subtasks() {
    let mock_server = MockServer::start().await;

    // Response includes the updated task with new subtasks
    let response_body = r##"{
        "id": "task123",
        "projectId": "proj456",
        "title": "Existing task",
        "isAllDay": false,
        "content": "",
        "priority": 0,
        "status": 0,
        "tags": [],
        "items": [
            {"id": "new_sub", "title": "New subtask", "status": 0, "completedTime": 0, "isAllDay": false, "sortOrder": 0, "timeZone": ""}
        ],
        "reminders": [],
        "sortOrder": 0,
        "timeZone": ""
    }"##;

    Mock::given(method("POST"))
        .and(path("/task/task123"))
        .and(bearer_token("test_token"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let request = UpdateTaskRequest {
        id: "task123".to_string(),
        project_id: "proj456".to_string(),
        title: None,
        content: None,
        is_all_day: None,
        start_date: None,
        due_date: None,
        priority: None,
        time_zone: None,
        tags: None,
        status: None,
        items: Some(vec![ChecklistItemRequest::new("New subtask")]),
    };

    let task = client.update_task("task123", &request).await.unwrap();

    assert_eq!(task.id, "task123");
    assert_eq!(task.items.len(), 1);
    assert_eq!(task.items[0].title, "New subtask");
}
