from fastapi import APIRouter, Depends, HTTPException, status, Query, UploadFile, File
from fastapi.security import HTTPBearer
from sqlalchemy.orm import Session
from typing import List, Optional
from datetime import datetime

from ..database import get_db
from ..models.item import Item
from ..models.user import User
from ..schemas.item import ItemCreate, ItemUpdate, ItemResponse, ItemListResponse, ItemSearchParams
from ..auth import get_current_user
from ..rust_api import validate_text_content, calculate_file_hash, filter_content_by_regex
from ..exceptions import ItemNotFoundError, UnauthorizedItemAccessError

router = APIRouter(
    prefix="/items",
    tags=["items"],
    responses={404: {"description": "Not found"}},
)

security = HTTPBearer()


@router.post("/", response_model=ItemResponse, status_code=status.HTTP_201_CREATED)
async def create_item(
    item_data: ItemCreate,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Create a new item"""
    # Validate content using Rust module
    if not validate_text_content(item_data.title):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid title content"
        )
    
    if item_data.description and not validate_text_content(item_data.description):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid description content"
        )
    
    # Create item
    db_item = Item(
        title=item_data.title,
        description=item_data.description,
        price=item_data.price,
        category=item_data.category,
        is_active=True,
        owner_id=current_user.id,
        created_at=datetime.utcnow(),
        updated_at=datetime.utcnow()
    )
    
    db.add(db_item)
    db.commit()
    db.refresh(db_item)
    
    return ItemResponse.from_orm(db_item)


@router.get("/", response_model=ItemListResponse)
async def list_items(
    skip: int = Query(0, ge=0),
    limit: int = Query(100, ge=1, le=1000),
    category: Optional[str] = None,
    min_price: Optional[float] = Query(None, ge=0),
    max_price: Optional[float] = Query(None, ge=0),
    search: Optional[str] = None,
    active_only: bool = True,
    owner_id: Optional[int] = None,
    db: Session = Depends(get_db)
):
    """List items with filtering and pagination"""
    query = db.query(Item)
    
    # Apply filters
    if active_only:
        query = query.filter(Item.is_active == True)
    
    if category:
        query = query.filter(Item.category == category)
    
    if min_price is not None:
        query = query.filter(Item.price >= min_price)
    
    if max_price is not None:
        query = query.filter(Item.price <= max_price)
    
    if owner_id:
        query = query.filter(Item.owner_id == owner_id)
    
    # Text search using Rust regex filtering
    if search:
        # Get all items first, then filter using Rust module
        all_items = query.all()
        filtered_items = []
        
        for item in all_items:
            search_text = f"{item.title} {item.description or ''}"
            if filter_content_by_regex(search_text, search):
                filtered_items.append(item)
        
        total = len(filtered_items)
        items = filtered_items[skip:skip + limit]
    else:
        total = query.count()
        items = query.offset(skip).limit(limit).all()
    
    return ItemListResponse(
        items=[ItemResponse.from_orm(item) for item in items],
        total=total,
        skip=skip,
        limit=limit
    )


@router.get("/my-items", response_model=ItemListResponse)
async def get_my_items(
    skip: int = Query(0, ge=0),
    limit: int = Query(100, ge=1, le=1000),
    active_only: bool = True,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Get current user's items"""
    query = db.query(Item).filter(Item.owner_id == current_user.id)
    
    if active_only:
        query = query.filter(Item.is_active == True)
    
    total = query.count()
    items = query.offset(skip).limit(limit).all()
    
    return ItemListResponse(
        items=[ItemResponse.from_orm(item) for item in items],
        total=total,
        skip=skip,
        limit=limit
    )


@router.get("/categories", response_model=List[str])
async def get_categories(
    db: Session = Depends(get_db)
):
    """Get all available item categories"""
    categories = db.query(Item.category).filter(
        Item.category.isnot(None),
        Item.is_active == True
    ).distinct().all()
    
    return [cat[0] for cat in categories if cat[0]]


@router.get("/{item_id}", response_model=ItemResponse)
async def get_item(
    item_id: int,
    db: Session = Depends(get_db)
):
    """Get item by ID"""
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise ItemNotFoundError(f"Item with id {item_id} not found")
    
    return ItemResponse.from_orm(item)


@router.put("/{item_id}", response_model=ItemResponse)
async def update_item(
    item_id: int,
    item_update: ItemUpdate,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Update item information"""
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise ItemNotFoundError(f"Item with id {item_id} not found")
    
    # Check ownership
    if item.owner_id != current_user.id and not current_user.is_superuser:
        raise UnauthorizedItemAccessError("Not authorized to update this item")
    
    update_data = item_update.dict(exclude_unset=True)
    
    # Validate content if being updated
    if "title" in update_data:
        if not validate_text_content(update_data["title"]):
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Invalid title content"
            )
    
    if "description" in update_data and update_data["description"]:
        if not validate_text_content(update_data["description"]):
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Invalid description content"
            )
    
    # Update item fields
    for field, value in update_data.items():
        setattr(item, field, value)
    
    item.updated_at = datetime.utcnow()
    
    db.commit()
    db.refresh(item)
    
    return ItemResponse.from_orm(item)


@router.delete("/{item_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_item(
    item_id: int,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Delete item (soft delete by deactivating)"""
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise ItemNotFoundError(f"Item with id {item_id} not found")
    
    # Check ownership
    if item.owner_id != current_user.id and not current_user.is_superuser:
        raise UnauthorizedItemAccessError("Not authorized to delete this item")
    
    # Soft delete by deactivating
    item.is_active = False
    item.updated_at = datetime.utcnow()
    
    db.commit()


@router.post("/{item_id}/activate", response_model=ItemResponse)
async def activate_item(
    item_id: int,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Activate deactivated item"""
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise ItemNotFoundError(f"Item with id {item_id} not found")
    
    # Check ownership
    if item.owner_id != current_user.id and not current_user.is_superuser:
        raise UnauthorizedItemAccessError("Not authorized to activate this item")
    
    item.is_active = True
    item.updated_at = datetime.utcnow()
    
    db.commit()
    db.refresh(item)
    
    return ItemResponse.from_orm(item)


@router.post("/{item_id}/upload-image")
async def upload_item_image(
    item_id: int,
    file: UploadFile = File(...),
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Upload image for item"""
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise ItemNotFoundError(f"Item with id {item_id} not found")
    
    # Check ownership
    if item.owner_id != current_user.id and not current_user.is_superuser:
        raise UnauthorizedItemAccessError("Not authorized to upload image for this item")
    
    # Validate file type
    if not file.content_type or not file.content_type.startswith("image/"):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="File must be an image"
        )
    
    # Read file content
    file_content = await file.read()
    
    # Calculate file hash using Rust module for integrity
    file_hash = calculate_file_hash(file_content)
    
    # Here you would typically save the file to storage (S3, local filesystem, etc.)
    # For this example, we'll just store the hash and filename
    
    # Update item with image info
    item.image_filename = file.filename
    item.image_hash = file_hash
    item.updated_at = datetime.utcnow()
    
    db.commit()
    
    return {
        "message": "Image uploaded successfully",
        "filename": file.filename,
        "hash": file_hash
    }


@router.post("/search", response_model=ItemListResponse)
async def search_items(
    search_params: ItemSearchParams,
    skip: int = Query(0, ge=0),
    limit: int = Query(100, ge=1, le=1000),
    db: Session = Depends(get_db)
):
    """Advanced item search with multiple criteria"""
    query = db.query(Item).filter(Item.is_active == True)
    
    # Apply search filters
    if search_params.category:
        query = query.filter(Item.category == search_params.category)
    
    if search_params.min_price is not None:
        query = query.filter(Item.price >= search_params.min_price)
    
    if search_params.max_price is not None:
        query = query.filter(Item.price <= search_params.max_price)
    
    if search_params.owner_id:
        query = query.filter(Item.owner_id == search_params.owner_id)
    
    # Text search with regex
    if search_params.text_search:
        all_items = query.all()
        filtered_items = []
        
        for item in all_items:
            search_text = f"{item.title} {item.description or ''}"
            if filter_content_by_regex(search_text, search_params.text_search):
                filtered_items.append(item)
        
        total = len(filtered_items)
        items = filtered_items[skip:skip + limit]
    else:
        total = query.count()
        items = query.offset(skip).limit(limit).all()
    
    return ItemListResponse(
        items=[ItemResponse.from_orm(item) for item in items],
        total=total,
        skip=skip,
        limit=limit
    )