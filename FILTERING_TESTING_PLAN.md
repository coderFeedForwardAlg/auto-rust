# Testing Plan for WHERE Clause Filtering

## Manual Testing Steps

1. Generate a new API project with a simple table (e.g., users with name, email fields)
2. Start the generated API server
3. Test the following curl commands:

### Test 1: Basic filtering
```bash
curl "http://localhost:8081/get_users?name=John"
```

### Test 2: Multiple filters
```bash
curl "http://localhost:8081/get_users?name=John&email=john@example.com"
```

### Test 3: Filtering with ordering
```bash
curl "http://localhost:8081/get_users?name=John&order_by=email&direction=desc"
```

### Test 4: Invalid field name (should return 400)
```bash
curl "http://localhost:8081/get_users?invalid_field=John"
```

### Test 5: Special characters in values
```bash
curl "http://localhost:8081/get_users?name=John+Doe"
```

## Expected Results

1. **Test 1**: Should return all users with name "John"
2. **Test 2**: Should return users with both name "John" AND email "john@example.com"
3. **Test 3**: Should return users with name "John", ordered by email descending
4. **Test 4**: Should return HTTP 400 Bad Request
5. **Test 5**: Should return users with name "John+Doe" (literal plus sign)

## Automated Test Plan

Add the following tests to the generated test suite:

1. **test_filtering_single_field** - Test single field filtering
2. **test_filtering_multiple_fields** - Test multiple field filtering
3. **test_filtering_with_ordering** - Test filtering combined with ordering
4. **test_filtering_invalid_field** - Test invalid field name handling
5. **test_filtering_special_characters** - Test special characters in values

## Implementation Verification

Check that the generated code includes:

1. HashMap import in the generated main.rs
2. Updated QueryParams struct with #[serde(flatten)] filters
3. Proper SQL parameter binding in the data layer function
4. Field name validation to prevent SQL injection
5. Correct WHERE clause generation with AND logic