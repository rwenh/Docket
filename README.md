# Task Manager API

Full CRUD REST API with JWT authentication, PostgreSQL, and filtering/pagination.

**Stack**: FastAPI · PostgreSQL · SQLAlchemy 2.0 · Alembic · python-jose · passlib

---

## Project Structure

```
task-api/
├── app/
│   ├── core/
│   │   ├── config.py       # Pydantic settings (reads .env)
│   │   ├── deps.py         # FastAPI dependencies (DB session, current user)
│   │   └── security.py     # JWT + password hashing
│   ├── db/
│   │   └── session.py      # SQLAlchemy engine + Base
│   ├── models/
│   │   ├── user.py         # User ORM model
│   │   └── task.py         # Task ORM model (status/priority enums)
│   ├── routers/
│   │   ├── auth.py         # POST /auth/register, POST /auth/login
│   │   └── tasks.py        # Full CRUD + filter + pagination
│   ├── schemas/
│   │   ├── user.py         # UserCreate, UserOut, Token
│   │   └── task.py         # TaskCreate, TaskUpdate, TaskOut, PaginatedTasks
│   └── main.py             # App factory, middleware, router registration
├── tests/
│   └── test_main.py        # Pytest suite (SQLite in-memory)
├── alembic/
│   └── env.py
├── docker-compose.yml
├── Dockerfile
└── requirements.txt
```

---

## Quick Start (local)

```bash
# 1. Clone & install
python -m venv venv && source venv/bin/activate
pip install -r requirements.txt

# 2. Configure
cp .env.example .env  # edit DATABASE_URL and SECRET_KEY

# 3. Start PostgreSQL (Docker)
docker compose up db -d

# 4. Run migrations
alembic upgrade head

# 5. Start the server
uvicorn app.main:app --reload
```

Swagger UI → http://localhost:8000/docs

---

## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | /auth/register | No | Register new user |
| POST | /auth/login | No | Login → JWT token |
| GET | /tasks | ✓ | List tasks (filter + paginate) |
| POST | /tasks | ✓ | Create task |
| GET | /tasks/{id} | ✓ | Get single task |
| PATCH | /tasks/{id} | ✓ | Partial update |
| DELETE | /tasks/{id} | ✓ | Delete task |

### Filtering & Pagination

```
GET /tasks?status=todo&priority=high&page=1&page_size=20
```

---

## Running Tests

```bash
pytest tests/ -v
```

Tests use an in-memory SQLite DB — no PostgreSQL needed.

---

## Migrations

```bash
# Generate a new migration after model changes
alembic revision --autogenerate -m "describe change"

# Apply
alembic upgrade head

# Roll back one step
alembic downgrade -1
```

---

## Deploying to Railway / Render

1. Push repo to GitHub
2. Create a new project → connect repo
3. Add a PostgreSQL plugin/database
4. Set environment variables: `DATABASE_URL`, `SECRET_KEY`
5. Set start command: `alembic upgrade head && uvicorn app.main:app --host 0.0.0.0 --port $PORT`

---

## Things to Add (stretch goals)

- [ ] Task due-date reminders (background tasks with APScheduler)
- [ ] Soft delete (`deleted_at` column)
- [ ] Admin role with access to all users' tasks
- [ ] Rate limiting (slowapi)
- [ ] Refresh tokens
