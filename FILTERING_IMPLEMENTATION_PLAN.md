# Implementation Plan: Adding WHERE Clause Filtering to Generated APIs

## Current State Analysis

Looking at the current `add_get_all_func` in `add_functions.rs`, we need to modify:

1. The `QueryParams` struct to capture filter parameters
2. The data layer function to generate dynamic WHERE clauses
3. The SQL query construction logic

## Detailed Implementation Steps

### Step 1: Update QueryParams Struct

Current struct:
```rust
#[derive(Deserialize)]
struct {row_name}QueryParams {
    order_by: Option<String>,
    direction: Option<String>, // "asc" or "desc"
}
```

New struct with filtering:
```rust
#[derive(Deserialize)]
struct {row_name}QueryParams {
    order_by: Option<String>,
    direction: Option<String>, // "asc" or "desc"
    #[serde(flatten)]
    filters: std::collections::HashMap<String, String>,
}
```

We'll need to add `use std::collections::HashMap;` to the imports.

### Step 2: Modify Data Layer Function

Current data function (simplified):
```rust
pub async fn data_{func_name}(
    extract::State(pool): extract::State<PgPool>,
    query_params: axum::extract::Query<{row_name}QueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut query = "SELECT * FROM {row_name}".to_owned();
    // Ordering logic here...
    let q = sqlx::query_as::<_, {struct_name}>(&query);
    // Execution and response handling...
}
```

New data function with filtering:
```rust
pub async fn data_{func_name}(
    extract::State(pool): extract::State<PgPool>,
    query_params: axum::extract::Query<{row_name}QueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut query = "SELECT * FROM {row_name}".to_owned();
    let mut sql_params: Vec<String> = Vec::new();
    let mut param_index = 1;
    
    // Handle filters
    if !query_params.filters.is_empty() {
        let mut where_conditions: Vec<String> = Vec::new();
        
        for (field, value) in &query_params.filters {
            // Skip ordering parameters
            if field == "order_by" || field == "direction" {
                continue;
            }
            
            // Validate field name to prevent SQL injection
            if is_valid_field_name(field) {
                where_conditions.push(format!("{} = ${}", field, param_index));
                sql_params.push(value.clone());
                param_index += 1;
            } else {
                return Err((StatusCode::BAD_REQUEST, format!("Invalid field name: {}", field)));
            }
        }
        
        if !where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", where_conditions.join(" AND ")));
        }
    }
    
    // Handle ordering (existing logic)
    if let Some(order_by) = &query_params.order_by {
        // ... existing ordering logic
    }
    
    // Execute query with parameters
    let mut query_builder = sqlx::query_as::<_, {struct_name}>(&query);
    for param in &sql_params {
        query_builder = query_builder.bind(param);
    }
    
    let elemints: Vec<{struct_name}> = query_builder.fetch_all(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;
    
    // ... rest of function unchanged
}
```

### Step 3: Add Helper Function

We need to add the field validation helper:
```rust
fn is_valid_field_name(field: &str) -> bool {
    // Only allow alphanumeric characters and underscores
    field.chars().all(|c| c.is_alphanumeric() || c == '_')
}
```

### Step 4: Update Imports

We need to add to the existing imports:
```rust
use std::collections::HashMap;
```

## Implementation in add_functions.rs

### Modification 1: Add HashMap import
Add to the imports section:
```rust
use std::collections::HashMap;
```

### Modification 2: Update QueryParams struct generation
Change:
```rust
#[derive(Deserialize)]
struct {row_name}QueryParams {
    order_by: Option<String>,
    direction: Option<String>, // "asc" or "desc"
}
```

To:
```rust
#[derive(Deserialize)]
struct {row_name}QueryParams {
    order_by: Option<String>,
    direction: Option<String>, // "asc" or "desc"
    #[serde(flatten)]
    filters: HashMap<String, String>,
}
```

### Modification 3: Update data function
Replace the data function generation with the enhanced version that includes:

1. Vector for SQL parameters
2. Filter processing logic
3. WHERE clause generation
4. Parameter binding
5. Field name validation

## Testing Considerations

### Unit Tests to Add
1. Field name validation function
2. SQL query generation with various filter combinations
3. Parameter binding correctness

### Integration Tests to Add
1. Single filter: `GET /get_users?name=John`
2. Multiple filters: `GET /get_users?name=John&email=john@example.com`
3. Filters with ordering: `GET /get_users?name=John&order_by=email`
4. Invalid field names: `GET /get_users?invalid_field=John` (should return 400)
5. Special characters in values: `GET /get_users?name=John+Doe`

## Security Considerations

1. **Field Name Validation**: Only allow alphanumeric and underscores
2. **Parameter Binding**: Use SQLx's bind() method for all user values
3. **Filter Whitelisting**: Consider implementing table-specific field whitelists
4. **Error Handling**: Return appropriate HTTP status codes for invalid input

## Example Generated Code

After implementation, the generated API would support:

```bash
# Filter by name
curl "http://localhost:8081/get_users?name=John"

# Filter by multiple fields
curl "http://localhost:8081/get_users?name=John&email=john@example.com"

# Filter with ordering
curl "http://localhost:8081/get_users?name=John&order_by=email&direction=desc"

# Complex filtering
curl "http://localhost:8081/get_users?status=active&department=engineering"
```

## Rollout Steps

1. Add HashMap import to add_functions.rs
2. Modify QueryParams struct generation
3. Enhance data layer function generation with filtering logic
4. Add field validation helper function
5. Test with sample API
6. Update documentation
7. Add tests to auto-generated test suite

## Future Enhancements

1. Support for other operators (!=, >, <, etc.)
2. OR logic between filters
3. Regular expression matching
4. Range queries
5. IN clause support