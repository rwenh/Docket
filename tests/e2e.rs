//! End-to-end smoke test. Spawns the real, compiled server binary against
//! the local Postgres instance and drives it purely over HTTP — no internal
//! function calls, no in-process router shortcuts. Mirrors the black-box
//! spirit of the pexpect-driven CLI smoke tests from the password manager
//! project, just with an HTTP client standing in for a pty.

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use serde_json::{json, Value};

struct ServerGuard(Child);

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn wait_until_healthy(client: &reqwest::Client) {
    for _ in 0..50 {
        if let Ok(resp) = client.get("http://127.0.0.1:8000/health").send().await {
            if resp.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("server did not become healthy in time");
}

#[tokio::test]
async fn full_api_flow() {
    let bin = env!("CARGO_BIN_EXE_task-manager");
    let child = Command::new(bin)
        .env(
            "DATABASE_URL",
            "postgresql://postgres:postgres@localhost:5432/taskdb",
        )
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn server binary");
    let _guard = ServerGuard(child);

    let client = reqwest::Client::new();
    wait_until_healthy(&client).await;

    // Unique email per run so repeated test runs don't collide on the
    // "already registered" check.
    let email = format!(
        "smoketest+{}@example.com",
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    );

    // ── register ─────────────────────────────────────────────────────────
    let resp = client
        .post("http://127.0.0.1:8000/auth/register")
        .json(&json!({ "email": email, "password": "hunter2hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let user: Value = resp.json().await.unwrap();
    assert_eq!(user["email"], email);
    assert_eq!(user["is_active"], true);

    // duplicate registration -> 400
    let resp = client
        .post("http://127.0.0.1:8000/auth/register")
        .json(&json!({ "email": email, "password": "hunter2hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // ── login ────────────────────────────────────────────────────────────
    let resp = client
        .post("http://127.0.0.1:8000/auth/login")
        .form(&[("username", email.as_str()), ("password", "wrong-password")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    let resp = client
        .post("http://127.0.0.1:8000/auth/login")
        .form(&[("username", email.as_str()), ("password", "hunter2hunter2")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let token_body: Value = resp.json().await.unwrap();
    let token = token_body["access_token"].as_str().unwrap().to_string();
    assert_eq!(token_body["token_type"], "bearer");

    // ── unauthenticated access -> 401 ───────────────────────────────────
    let resp = client.get("http://127.0.0.1:8000/tasks").send().await.unwrap();
    assert_eq!(resp.status(), 401);

    // ── create task ──────────────────────────────────────────────────────
    let resp = client
        .post("http://127.0.0.1:8000/tasks")
        .bearer_auth(&token)
        .json(&json!({
            "title": "Write the Rust port",
            "description": "Mirror the FastAPI app",
            "priority": "high"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let task: Value = resp.json().await.unwrap();
    assert_eq!(task["status"], "todo"); // default applied
    assert_eq!(task["priority"], "high");
    assert_eq!(task["description"], "Mirror the FastAPI app");
    let task_id = task["id"].as_i64().unwrap();
    let created_at = task["created_at"].as_str().unwrap().to_string();

    // two more tasks, to exercise pagination + filtering
    for (title, priority) in [("Second task", "low"), ("Third task", "high")] {
        let resp = client
            .post("http://127.0.0.1:8000/tasks")
            .bearer_auth(&token)
            .json(&json!({ "title": title, "priority": priority }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
    }

    // ── list (no trailing slash on the nest nested at "/") ──────────────
    let resp = client
        .get("http://127.0.0.1:8000/tasks")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let page: Value = resp.json().await.unwrap();
    assert_eq!(page["total"], 3);
    assert_eq!(page["page"], 1);
    assert_eq!(page["page_size"], 20);
    assert_eq!(page["pages"], 1);
    assert_eq!(page["items"].as_array().unwrap().len(), 3);

    // ── filter by priority ───────────────────────────────────────────────
    let resp = client
        .get("http://127.0.0.1:8000/tasks?priority=high")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let filtered: Value = resp.json().await.unwrap();
    assert_eq!(filtered["total"], 2);

    // ── query param validation -> 422 ────────────────────────────────────
    let resp = client
        .get("http://127.0.0.1:8000/tasks?page=0")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);

    let resp = client
        .get("http://127.0.0.1:8000/tasks?page_size=101")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);

    // ── get by id ─────────────────────────────────────────────────────────
    let resp = client
        .get(format!("http://127.0.0.1:8000/tasks/{task_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // ── get nonexistent -> 404 ──────────────────────────────────────────
    let resp = client
        .get("http://127.0.0.1:8000/tasks/999999999")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    // ── patch: change status, explicitly null out description, leave
    //    title/priority untouched (proves partial-update semantics) ──────
    let resp = client
        .patch(format!("http://127.0.0.1:8000/tasks/{task_id}"))
        .bearer_auth(&token)
        .json(&json!({ "status": "in_progress", "description": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["status"], "in_progress");
    assert!(updated["description"].is_null());
    assert_eq!(updated["title"], "Write the Rust port"); // untouched
    assert_eq!(updated["priority"], "high"); // untouched
    assert_ne!(updated["updated_at"], created_at); // touched on write

    // ── delete ───────────────────────────────────────────────────────────
    let resp = client
        .delete(format!("http://127.0.0.1:8000/tasks/{task_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let resp = client
        .get(format!("http://127.0.0.1:8000/tasks/{task_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
