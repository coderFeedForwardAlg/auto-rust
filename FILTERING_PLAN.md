# Plan: Adding WHERE Clause Filtering to Generated APIs

## Overview
This plan outlines how to extend the generated APIs to support dynamic WHERE clause filtering based on URL query parameters. Users should be able to filter data by specifying field-value pairs in the query string, with support for multiple filters combined with AND logic.

## Requirements
1. Support filtering by any number of field-value pairs
2. Generate WHERE clauses dynamically based on query parameters
3. Only support equality comparisons (=) for simplicity
4. Prevent SQL injection through proper parameterization
5. Handle different data types appropriately
6. Return appropriate error responses for invalid parameters

## Implementation Approach

### 1. Query Parameter Structure
Users will specify filters using query parameters in the format:
```
GET /get_users?name=John&email=john@example.com&age=30
```

This should generate a SQL query like:
```sql
SELECT * FROM users WHERE name = $1 AND email = $2 AND age = $3
```

### 2. Data Structure for Filters
Create a new struct to represent filter parameters:

```rust
#[derive(Deserialize)]
struct FilterParams {
    // Standard ordering parameters
    order_by: Option<String>,
    direction: Option<String>,
    // All other query parameters become filters
    #[serde(flatten)]
    filters: HashMap<String, String>,
}
```

### 3. SQL Query Generation
Modify the data layer function to:
1. Extract filter parameters from the query string
2. Validate field names to prevent SQL injection
3. Generate dynamic WHERE clause
4. Bind parameter values safely
5. Combine with existing ORDER BY logic

### 4. Parameter Binding
Use SQLx's parameter binding to safely inject values:
```rust
let mut query = "SELECT * FROM users".to_string();
let mut params: Vec<&str> = Vec::new();

if !filters.is_empty() {
    let where_conditions: Vec<String> = filters
        .iter()
        .enumerate()
        .map(|(i, (field, value))| {
            params.push(value);
            format!("{} = ${}", field, i + 1)
        })
        .collect();
    
    query.push_str(&format!(" WHERE {}", where_conditions.join(" AND ")));
}
```

## Detailed Implementation Steps

### Step 1: Modify Query Parameter Struct
Update the generated query parameter structs to include filter support:

```rust
#[derive(Deserialize)]
struct UsersQueryParams {
    order_by: Option<String>,
    direction: Option<String>,
    // This will capture all other query parameters as filters
    #[serde(flatten)]
    filters: std::collections::HashMap<String, String>,
}
```

### Step 2: Enhance Data Layer Function
Update the data_get_* functions to handle dynamic filtering:

```rust
pub async fn data_get_users(
    extract::State(pool): extract::State<PgPool>,
    query_params: axum::extract::Query<UsersQueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut query = "SELECT * FROM users".to_owned();
    let mut sql_params: Vec<String> = Vec::new();
    let mut param_index = 1;
    
    // Handle filters
    if !query_params.filters.is_empty() {
        let mut where_conditions: Vec<String> = Vec::new();
        
        for (field, value) in &query_params.filters {
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
        if is_valid_field_name(order_by) {
            let direction = match &query_params.direction {
                Some(dir) if dir.to_lowercase() == "desc" => "DESC",
                _ => "ASC",
            };
            query.push_str(&format!(" ORDER BY {} {}", order_by, direction));
        } else {
            return Err((StatusCode::BAD_REQUEST, "Invalid order_by parameter".to_string()));
        }
    }
    
    // Execute query with parameters
    let mut query_builder = sqlx::query_as::<_, Users>(&query);
    for param in &sql_params {
        query_builder = query_builder.bind(param);
    }
    
    let elemints: Vec<Users> = query_builder.fetch_all(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;
    
    // ... rest of function
}
```

### Step 3: Add Field Name Validation
Implement a helper function to validate field names:

```rust
fn is_valid_field_name(field: &str) -> bool {
    // Only allow alphanumeric characters and underscores
    field.chars().all(|c| c.is_alphanumeric() || c == '_')
}
```

### Step 4: Update Code Generation
Modify the `add_get_all_func` function in `add_functions.rs` to generate the enhanced code:

1. Update the `FilterParams` struct definition
2. Enhance the data layer function generation
3. Add the validation helper function

## Security Considerations

1. **SQL Injection Prevention**:
   - Validate all field names against allowed characters
   - Use parameterized queries for all values
   - Never directly interpolate user input into SQL strings

2. **Input Validation**:
   - Limit field names to alphanumeric characters and underscores
   - Consider implementing a whitelist of allowed fields per table
   - Validate data types where possible

## Example Usage

After implementation, users can filter data like this:

```
# Filter by single field
GET /get_users?name=John

# Filter by multiple fields
GET /get_users?name=John&email=john@example.com

# Filter with ordering
GET /get_users?name=John&order_by=email&direction=desc

# Complex filtering
GET /get_users?status=active&department=engineering&city=New_York
```

## Testing Plan

1. **Unit Tests**:
   - Test field name validation
   - Test SQL query generation with various filter combinations
   - Test parameter binding

2. **Integration Tests**:
   - Test single filter queries
   - Test multiple filter queries
   - Test combined filtering and ordering
   - Test error cases (invalid field names, etc.)

3. **Security Tests**:
   - Test SQL injection attempts
   - Test special character handling
   - Test boundary conditions

## Rollout Strategy

1. Implement the core functionality in the code generation
2. Test with a sample API
3. Add comprehensive error handling
4. Document the feature for API users
5. Add integration tests to the auto-generated test suite

## Future Enhancements

1. Support for other comparison operators (!=, >, <, etc.)
2. Support for OR logic between filters
3. Regular expression matching
4. Range queries
5. IN clause support for multiple values