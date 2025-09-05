# Verification Plan for HashMap Import

## Steps to Verify

1. Generate a new API project
2. Check that the generated main.rs includes the HashMap import
3. Test the filtering functionality

## Expected Results

1. The generated main.rs should include:
   ```rust
   use std::collections::HashMap;
   ```

2. The generated QueryParams structs should include:
   ```rust
   #[derive(Deserialize)]
   struct {TableName}QueryParams {
       order_by: Option<String>,
       direction: Option<String>, // "asc" or "desc"
       #[serde(flatten)]
       filters: HashMap<String, String>,
   }
   ```

3. The data layer functions should handle dynamic filtering with WHERE clauses

## Test Commands

After generating a new API:

```bash
# Check for HashMap import
grep "use std::collections::HashMap;" src/main.rs

# Check for filters in QueryParams struct
grep -A 5 "#\[serde(flatten)\]" src/main.rs

# Test filtering
curl "http://localhost:8081/get_{endpoint}?{field}={value}"
```