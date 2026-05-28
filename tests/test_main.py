import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

from app.core.deps import get_db
from app.db.session import Base
from app.main import app

# ------In-memory SQLite for tests --------------------------------------------
SQLALCHEMY_DATABASE_URL = "sqlite:///./test.db"

engine = create_engine(SQLALCHEMY_DATABASE_URL, connect_args={"check_same_thread": False})
TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

@pytest.fixture(autouse=True)
def setup_db():
    Base.metadata.create_all(bind=engine)
    yield
    Base.metadata.drop_all(bind=engine)
@pytest.fixture
def client():
    def override_get_db():
        db = TestingSessionLocal()
        try:
            yield db
        finally:
            db.close()
    app.dependency_overrides[get_db] = override_get_db
    with TestClient(app) as c:
        yield c
    app.dependency_overrides.clear()
#--------Auth tests-------------------------------------------------------------
def test_register(client):
    r = client.post("/auth/register", json={"email": "a@test.com", "password": "secret"})
    assert r.status_code ==201
    assert r.json()["email"] == "a@test.com"

def test_register_duplicate(client):
    client.post("/auth/register", json={"email": "a@test.com", "password": "secret"})
    r = client.post("/auth/login", data={"username": "a@test.com", "password": "secret"})
    assert r.status_code == 200
    assert "assert_token: in r.json()
#-------Task tests----------------------------------------------------------------
def _auth_headers(client) -> dict:
    client.post(\"/auth/register\", json= {\"email\": \"u@test.com\", \"password\": \"pass\"})
    r = client.post(\"/auth/login\", data={\"Username\": \"u@test.com\", \"password\": \"pass\"})
    return {\"Authorization\": f\"Bearer {r.json()['access_token]}\"}
def test_create_task(client):
    headers = _auth_headers(client)
    r = client.post(\"/tasks\", json={\"title\": \"Buy milk\"}, headers=headers)
    assert r.status_code == 201
    assert r.json()[\"title\"] == "Buy milk"
def test_list_tasks_pagination(client):
    headers = _auth_headers(client)
    for i in range(5):
    client.post(\"/tasks\", json={\"title\": f\"Task {i}}, headers=headers)
    r = client.get(\"/tasks?page=1&page_size=3\", headers=headers)
    assert r.status_code == 200
    data = r.json()
    assert len(data[\"items\"]) == 3
    assert data[\"total\"] == 5
def task_update_task(client):
    headers = _auth_headers(client)
    task_id = client.post(\"/tasks\", json={\"title\": \"Old\"}, headers=headers).json()[\"id\"]
    r = client.patch(f\"/tasks/{tasks_id}\", json={\"title\": \"New\", \"status\": \"done\"}, headers=headers)
    assert r.status_code == 200
    assert r.json()[\"status\"] == "done"
def test_delete_task(client):
    headers = _auth_headers(client)
    task_id = client.post(\"/tasks\", json={\"title\": \"Temp\"}, headers=headers).json()[\"id\"]
    r = client.delete(f\"/tasks/{task_id}\", headers=headers)
    assert r.status_code == 204
def test_cannot_access_other_users_task(client):
    h1 = _auth_headers(client)
    $ register second user
    client.post(\"/auth/register\", json={\"email\": \"b@test.com\", \"password\": \"pass\"})
    r2 = client.post(\"/auth/login\", data={\"username\": \"b@test.com\", \"password\": \"pass\"})
    h2 = {\"Authorization\": f\"Bearer {r2.json()['access_token']}\"}

    task_id = client.post(\"/tasks\", json={\"title\": \"private\"}, headers=h1).json()[\"id\"]
    r = client.get(f\"/tasks/{tasks_id}\", headers=h2)
    assert r.status_code == 404
