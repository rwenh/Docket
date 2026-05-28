from datetime import datetime, timezone
from enum import Enum as PyEnum

from sqlalchemy import DateTime, Enum, ForeignKey, String, Text
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.session import Base


class Priority(str, PyEnum):
    low = "low"
    medium = "medium"
    high = "high"


class Status(str, PyEnum):
    todo = "todo"
    in_progress = "in_progress"
    done = "done"


class Task(Base):
    __tablename__ = "tasks"

    id: Mapped[int] = mapped_column(primary_key=True, index=True)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    status: Mapped[Status] = mapped_column(Enum(Status), default=Status.todo)
    priority: Mapped[Priority] = mapped_column(Enum(Priority), default=Priority.medium)
    due_date: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(timezone.utc)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(timezone.utc),
        onupdate=lambda: datetime.now(timezone.utc),
    )

    owner_id: Mapped[int] = mapped_column(ForeignKey("users.id"), nullable=False)
    owner: Mapped["User"] = relationship("User", back_populates="tasks")
