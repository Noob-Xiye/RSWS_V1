use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
    pub path: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// 分类树节点（带子分类）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTreeNode {
    #[serde(flatten)]
    pub category: Category,
    pub children: Vec<CategoryTreeNode>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Option<i64>>,
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
            r#"SELECT id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at
            FROM categories WHERE is_active = true ORDER BY sort_order"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(categories)
    }

    /// 查询所有分类（含已停用），管理员用
    pub async fn find_all_with_inactive(&self) -> Result<Vec<Category>, sqlx::Error> {
        let categories = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at
            FROM categories ORDER BY sort_order"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(categories)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Category>, sqlx::Error> {
        let category = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at
            FROM categories WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(category)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Category>, sqlx::Error> {
        let category = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at
            FROM categories WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(category)
    }

    /// 获取最大 sort_order 值
    pub async fn max_sort_order(&self) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<i32>>("SELECT MAX(sort_order) FROM categories")
            .fetch_one(&self.pool)
            .await?;
        Ok(result.unwrap_or(0))
    }

    pub async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        parent_id: Option<i64>,
        sort_order: i32,
    ) -> Result<Category, sqlx::Error> {
        let category = sqlx::query_as::<_, Category>(
            r#"INSERT INTO categories (name, description, parent_id, sort_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at"#,
        )
        .bind(name)
        .bind(description)
        .bind(parent_id)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await?;

        // Compute path with the actual id
        let actual_path = if let Some(pid) = parent_id {
            let parent = self.find_by_id(pid).await?;
            let base = parent
                .and_then(|p| p.path)
                .unwrap_or_else(|| format!("/{}", pid));
            format!("{}{}/", base, category.id)
        } else {
            format!("/{}/", category.id)
        };

        // Update path in DB
        let category = sqlx::query_as::<_, Category>(
            r#"UPDATE categories SET path = $1 WHERE id = $2
            RETURNING id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at"#,
        )
        .bind(&actual_path)
        .bind(category.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        description: Option<Option<&str>>,
        parent_id: Option<Option<i64>>,
        sort_order: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<Option<Category>, sqlx::Error> {
        let existing = self.find_by_id(id).await?;
        let cat = match existing {
            Some(c) => c,
            None => return Ok(None),
        };

        // Resolve new parent_id
        let new_parent_id = match &parent_id {
            Some(Some(pid)) => {
                if *pid == id {
                    return Err(sqlx::Error::RowNotFound);
                }
                let parent_cat = self.find_by_id(*pid).await?;
                if let Some(p) = &parent_cat {
                    if let Some(ref p_path) = p.path {
                        if p_path.contains(&format!("/{}/", id)) {
                            return Err(sqlx::Error::RowNotFound);
                        }
                    }
                }
                Some(*pid)
            }
            Some(None) => None,
            None => cat.parent_id,
        };

        let new_name = name.unwrap_or(&cat.name);
        let new_description = description.unwrap_or(cat.description.as_deref());
        let new_sort_order = sort_order.unwrap_or(cat.sort_order);
        let new_is_active = is_active.unwrap_or(cat.is_active);

        // Compute new path if parent changed
        let new_path = if parent_id.is_some() {
            match new_parent_id {
                Some(pid) => {
                    let parent = self.find_by_id(pid).await?;
                    let base = parent
                        .and_then(|p| p.path)
                        .unwrap_or_else(|| format!("/{}", pid));
                    format!("{}{}/", base, id)
                }
                None => format!("/{}/", id),
            }
        } else {
            cat.path.unwrap_or_else(|| format!("/{}/", id))
        };

        let category = sqlx::query_as::<_, Category>(
            r#"UPDATE categories SET name = $1, description = $2, parent_id = $3, path = $4, sort_order = $5, is_active = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at"#,
        )
        .bind(new_name)
        .bind(new_description)
        .bind(new_parent_id)
        .bind(&new_path)
        .bind(new_sort_order)
        .bind(new_is_active)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        // Rebuild descendant paths if parent changed
        if parent_id.is_some() {
            self.rebuild_descendant_paths(id, &new_path).await.ok();
        }

        Ok(category)
    }

    pub async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        // Children's parent_id will be set to NULL by ON DELETE SET NULL
        let result = sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() > 0 {
            // Rebuild paths for orphaned children
            self.rebuild_descendant_paths(id, &format!("/{}/", id))
                .await
                .ok();
        }
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

    /// 重建所有后代的 path
    async fn rebuild_descendant_paths(
        &self,
        parent_id: i64,
        parent_path: &str,
    ) -> Result<(), sqlx::Error> {
        let children: Vec<Category> = sqlx::query_as::<_, Category>(
            r#"SELECT id, name, description, parent_id, path, sort_order, is_active, created_at, updated_at
            FROM categories WHERE parent_id = $1"#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        for child in children {
            let child_path = format!("{}{}/", parent_path, child.id);
            sqlx::query("UPDATE categories SET path = $1 WHERE id = $2")
                .bind(&child_path)
                .bind(child.id)
                .execute(&self.pool)
                .await?;
            // Recursively rebuild
            Box::pin(self.rebuild_descendant_paths(child.id, &child_path)).await?;
        }
        Ok(())
    }

    /// 构建分类树（从扁平列表）
    pub fn build_tree(categories: &[Category]) -> Vec<CategoryTreeNode> {
        let mut root_nodes: Vec<CategoryTreeNode> = Vec::new();
        let mut node_map: std::collections::HashMap<i64, CategoryTreeNode> =
            std::collections::HashMap::new();

        // Create all nodes
        for cat in categories {
            node_map.insert(
                cat.id,
                CategoryTreeNode {
                    category: cat.clone(),
                    children: Vec::new(),
                },
            );
        }

        // Build tree
        let mut children_map: std::collections::HashMap<i64, Vec<CategoryTreeNode>> =
            std::collections::HashMap::new();
        let mut ids: Vec<i64> = node_map.keys().cloned().collect();
        ids.sort();

        for id in ids {
            if let Some(node) = node_map.remove(&id) {
                let pid = node.category.parent_id;
                match pid {
                    Some(pid) => {
                        children_map.entry(pid).or_default().push(node);
                    }
                    None => {
                        root_nodes.push(node);
                    }
                }
            }
        }

        // Attach children to parents
        fn attach_children(
            nodes: &mut [CategoryTreeNode],
            children_map: &std::collections::HashMap<i64, Vec<CategoryTreeNode>>,
        ) {
            for node in nodes.iter_mut() {
                if let Some(children) = children_map.get(&node.category.id) {
                    node.children = children.clone();
                    attach_children(&mut node.children, children_map);
                }
            }
        }
        attach_children(&mut root_nodes, &children_map);

        root_nodes
    }

    /// 获取指定父分类下的最大 sort_order
    pub async fn max_sort_order_under_parent(
        &self,
        parent_id: Option<i64>,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(sort_order) FROM categories WHERE parent_id IS NOT DISTINCT FROM $1",
        )
        .bind(parent_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.unwrap_or(0))
    }
}
