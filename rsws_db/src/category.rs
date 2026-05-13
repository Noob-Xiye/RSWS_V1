use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

pub struct CategoryRepository {
    pool: PgPool,
}

impl CategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<Category>, sqlx::Error> {
        let categories = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, sort_order, is_active, created_at, updated_at
            FROM categories WHERE is_active = true ORDER BY sort_order"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(categories)
    }

    /// 查询所有分类（含已停用），管理员用
    pub async fn find_all_with_inactive(&self) -> Result<Vec<Category>, sqlx::Error> {
        let categories = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, sort_order, is_active, created_at, updated_at
            FROM categories ORDER BY sort_order"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(categories)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Category>, sqlx::Error> {
        let category = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, sort_order, is_active, created_at, updated_at
            FROM categories WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(category)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Category>, sqlx::Error> {
        let category = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, sort_order, is_active, created_at, updated_at
            FROM categories WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(category)
    }

    /// 获取最大 sort_order 值
    pub async fn max_sort_order(&self) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(sort_order) FROM categories",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.unwrap_or(0))
    }

    pub async fn create(&self, name: &str, description: Option<&str>, sort_order: i32) -> Result<Category, sqlx::Error> {
        let category = sqlx::query_as::<_, Category>(
            r#"INSERT INTO categories (name, description, sort_order)
            VALUES ($1, $2, $3)
            RETURNING id, name, description, sort_order, is_active, created_at, updated_at"#,
        )
        .bind(name)
        .bind(description)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await?;
        Ok(category)
    }

    pub async fn update(&self, id: i64, name: Option<&str>, description: Option<Option<&str>>, sort_order: Option<i32>, is_active: Option<bool>) -> Result<Option<Category>, sqlx::Error> {
        let existing = self.find_by_id(id).await?;
        let cat = match existing {
            Some(c) => c,
            None => return Ok(None),
        };

        let new_name = name.unwrap_or(&cat.name);
        let new_description = description.unwrap_or(cat.description.as_deref());
        let new_sort_order = sort_order.unwrap_or(cat.sort_order);
        let new_is_active = is_active.unwrap_or(cat.is_active);

        let category = sqlx::query_as::<_, Category>(
            r#"UPDATE categories SET name = $1, description = $2, sort_order = $3, is_active = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING id, name, description, sort_order, is_active, created_at, updated_at"#,
        )
        .bind(new_name)
        .bind(new_description)
        .bind(new_sort_order)
        .bind(new_is_active)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(category)
    }

    pub async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 批量更新排序
    pub async fn batch_update_sort_order(&self, orders: &[(i64, i32)]) -> Result<(), sqlx::Error> {
        for &(id, sort_order) in orders {
            sqlx::query("UPDATE categories SET sort_order = $1, updated_at = NOW() WHERE id = $2")
                .bind(sort_order)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    /// 统计分类下的资源数量
    pub async fn count_resources(&self, category_id: i64) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM resources WHERE category_id = $1 AND is_active = true",
        )
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }
}
