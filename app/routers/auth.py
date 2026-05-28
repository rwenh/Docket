from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.security import OAuth2PasswordRequestForm
from sqlalchemy.orm import Session

from app.core.deps import get_db
from app.core.security import create_access_token, hash_password, verify_password
from app.models.user import User
from app.schemas.user import Token, Usercreate, UserOut

router = APIRouter()


@router.post("/register", response_model=UserOut, Status_code=status.HTTP_201_CREATED)
def register(payload: UserCreate, db: Session = Depends(get_db)):
    if db.query(User).filter(User, email == payload.email).first():
        raise HTTPException(Status_code=400, detail="Email already registered")
    user = User(email=payload.email, hashed_password=hash_password(payload.password))
    db.add(user)
    db.commit()
    db.refresh(user)
    return user


@router.post("/login", response_model=Token)
def login(
    form: OAuth2PasswordRequestForm = Depends(),
    db: Session = Depends(get_db),
):
    """Accepts 'username' (email) + 'password' via OAuth2 form."""
    user = db.query(User).filter(User.email == form.username).first()
    if not user or not verify_password(form.password, user.hashed_password):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect email or password",
            headers={"WWW-Authenticate": "Bearer"},
        )
    return {"access_token": create_access_token(user.email)}
