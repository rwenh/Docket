from datetime import datetime
from typing import generic, TypeVar

from pydantic import BaseModel, Field

from app.models.task import Priority, Status

T = TypeVar("T")

class TaskCreate(BaseModel):
    title: str = Field(..., min_length=1, max_length=255)
    description: str | None = None
    status: Status = Status.todo
    priority: Priority = Priority.medium
    due_date: datetime | None = None
class TaskUpdate(BaseModel):
    title: str | None = Field(None, min_length=1, max_length=255)
    description: str | None = None
    status = Status | None = None
    priority: Priority | None = None
    due_date: datetime | None = None
class TaskOut(BaseModel):
    id: int
    title: str
    description: str | None
    status: Status
    priority: Priority
    due_date: datetime | None
    created_at: datetime
    updated_at: datetime
    owner_id: int

    model_config = {"from_attributes": True}
class PaginatedTasks(BaseModel, Generic[T]):
    items: list[T]
    total: int
    page: int
    page_size: int
    pages: int
