# Large Complex Test Dataset Documentation

## Overview

This document describes the comprehensive test dataset (`test_large_complex.csv`) used to test the CSV import functionality, type inference, and data handling capabilities of the Fresh application.

## Dataset Specifications

### File Structure
- **Total Rows**: ~100,000 data rows
- **Headers**: 32 columns with realistic field names
- **File Size**: ~15-20 MB
- **Encoding**: UTF-8
- **Delimiter**: Comma (,)

### File Layout
```
# Large Complex Test Dataset
# This file contains ~100,000 rows of realistic data with various distributions and null patterns
# Used to test CSV import functionality, type inference, and data handling

# Fake header line to test header row selection
Fake Header Line - Ignore This

# Another fake line
Another fake line with some text

# Real headers start here
user_id,age,income,is_active,registration_date,last_login_time,premium_tier,country_code,score,rating,has_verified_email,username,created_at,login_count,subscription_type,payment_method,account_balance,monthly_spend,is_premium,email_domain,join_date,timezone,last_purchase_date,referral_source,device_type,os_version,app_version,location_lat,location_lng,session_duration,page_views,conversion_rate,churn_probability

# Data starts here - 100,000 rows with realistic patterns
1,25,45000,true,2023-01-15,14:30:25,premium,US,85.5,4.2,true,john_doe,2023-01-15T14:30:25.123Z,156,monthly,credit_card,1250.75,89.99,true,gmail.com,2023-01-15,UTC-5,2023-12-15T10:30:00.000Z,google,iphone,iOS 17.2,2.1.0,40.7128,-74.0060,1800,45,0.023,0.15
...
```

## Column Types and Data Patterns

### 1. Integer Columns
- **user_id**: Sequential integers (1-100,000)
- **age**: Realistic age distribution (18-75, normal distribution around 35)
- **income**: Realistic income distribution (20k-200k, log-normal distribution)
- **login_count**: Integer (0-500, exponential distribution)
- **session_duration**: Integer seconds (300-3600, exponential distribution)
- **page_views**: Integer (5-200, exponential distribution)

### 2. Float Columns
- **score**: Float (0.0-100.0, weighted towards 60-90)
- **rating**: Float (1.0-5.0, weighted towards 3-5)
- **account_balance**: Float (0.0-10000.0, realistic distribution)
- **monthly_spend**: Float (0.0-500.0, realistic distribution)
- **location_lat**: Float (-90.0 to 90.0, realistic coordinates)
- **location_lng**: Float (-180.0 to 180.0, realistic coordinates)
- **conversion_rate**: Float (0.0-0.1, realistic conversion rates)
- **churn_probability**: Float (0.0-1.0, realistic churn probabilities)

### 3. Boolean Columns
- **is_active**: Boolean (true/false with 2% null probability)
- **has_verified_email**: Boolean (true/false with 3% null probability)
- **is_premium**: Boolean (true/false with 2% null probability)

### 4. Date/Time Columns
- **registration_date**: Date (YYYY-MM-DD format, 2023 dates)
- **join_date**: Date (YYYY-MM-DD format, 2023 dates)
- **last_login_time**: Time (HH:MM:SS format)
- **created_at**: DateTime (ISO 8601 format with milliseconds)
- **last_purchase_date**: DateTime (ISO 8601 format with milliseconds)

### 5. Text/Categorical Columns
- **premium_tier**: Categorical (free, standard, premium, enterprise)
- **country_code**: Categorical (US, CA, UK, DE, FR, AU, JP, IT, BR, ES, etc.)
- **username**: Text (realistic usernames with 20% null probability)
- **subscription_type**: Categorical (monthly, annual, quarterly, weekly)
- **payment_method**: Categorical (credit_card, paypal, debit_card, etc.)
- **email_domain**: Categorical (gmail.com, outlook.com, yahoo.com, etc.)
- **timezone**: Categorical (UTC-8 to UTC+12)
- **referral_source**: Categorical (google, facebook, instagram, etc.)
- **device_type**: Categorical (iphone, android, desktop, tablet, smartwatch)
- **os_version**: Categorical (device-specific OS versions)
- **app_version**: Text (semantic versioning format)

## Null Value Patterns

### Null Value Types
- Empty strings (`""`)
- "null" (lowercase)
- "NULL" (uppercase)
- "-" (dash)
- "N/A" (not applicable)

### Null Probabilities by Column Type
- **Boolean columns**: 2-3% null probability
- **Integer columns**: 2% null probability
- **Float columns**: 3-15% null probability (higher for rates/probabilities)
- **Date/Time columns**: 1-5% null probability
- **Text columns**: 20% null probability for usernames, 0% for categorical

## Data Quality Features

### 1. Garbage Lines
- Every 10,000 rows includes a garbage line with random text
- Examples: "This is a garbage line that should be ignored", "ooOOoooo We end here.", "ERROR: Invalid data format"

### 2. Empty Rows
- Every 5,000 rows includes a completely empty row
- Tests handling of empty data

### 3. Realistic Distributions
- **Age**: Normal distribution around 35 years
- **Income**: Log-normal distribution (right-skewed)
- **Session duration**: Exponential distribution (most sessions 5-30 minutes)
- **Page views**: Exponential distribution (most users view 5-50 pages)
- **Conversion rates**: Realistic low rates (0-10%)
- **Churn probabilities**: Realistic distribution (0-100%)

### 4. Correlated Data
- **Device type** and **OS version** are correlated
- **Country** and **timezone** are loosely correlated
- **Income** and **premium tier** are correlated
- **Session duration** and **page views** are correlated

## Testing Objectives

### 1. Type Inference Testing
- **Integer detection**: Sequential IDs, realistic ages, counts
- **Float detection**: Scores, ratings, coordinates, rates
- **Boolean detection**: True/false values with various formats
- **Date detection**: Multiple date formats (YYYY-MM-DD, ISO 8601)
- **Time detection**: HH:MM:SS format
- **Text detection**: Categorical and free-form text

### 2. Null Value Handling
- **Empty strings**: Most common null type
- **"null" variations**: Lowercase, uppercase, dash
- **Mixed null types**: Different columns have different null patterns
- **Null in headers**: Some columns have null values in header row

### 3. Header Row Selection
- **Fake headers**: Multiple fake header lines to test selection
- **Real headers**: Start at row 5 (after fake lines)
- **Header validation**: Ensure correct header row is selected

### 4. Data Quality Testing
- **Garbage filtering**: Lines that should be ignored
- **Empty row handling**: Completely empty rows
- **Mixed data quality**: Some rows perfect, some with issues
- **Large file handling**: Performance with 100k+ rows

### 5. Performance Testing
- **Memory usage**: Large dataset memory consumption
- **Processing speed**: Time to process 100k rows
- **Type inference speed**: Time to infer types from large sample
- **Import speed**: Time to import entire dataset

### 6. Edge Cases
- **Quoted strings**: Some text fields contain quotes
- **Special characters**: Various special characters in text fields
- **Very long text**: Some text fields are quite long
- **Mixed encodings**: UTF-8 with various characters
- **Inconsistent data**: Some rows have missing columns

## Expected Type Inference Results

### Should be inferred as Integer:
- user_id, age, income, login_count, session_duration, page_views

### Should be inferred as Float:
- score, rating, account_balance, monthly_spend, location_lat, location_lng, conversion_rate, churn_probability

### Should be inferred as Boolean:
- is_active, has_verified_email, is_premium

### Should be inferred as Date:
- registration_date, join_date

### Should be inferred as Time:
- last_login_time

### Should be inferred as DateTime:
- created_at, last_purchase_date

### Should be inferred as Text:
- premium_tier, country_code, username, subscription_type, payment_method, email_domain, timezone, referral_source, device_type, os_version, app_version

## Usage Instructions

### 1. Generate the Dataset
```bash
python generate_large_test_data.py
```

### 2. Test in Fresh Application
1. Open Fresh application
2. Create new project from CSV
3. Select `test_large_complex.csv`
4. Set header row to 5 (after fake headers)
5. Test type inference and import

### 3. Expected Results
- All 32 columns should be detected
- Type inference should be accurate for all columns
- Import should complete without errors
- Performance should be reasonable (< 30 seconds for import)
- Memory usage should be manageable

## Troubleshooting

### Common Issues
1. **Memory errors**: Dataset is large, may need more RAM
2. **Type inference errors**: Check if all null patterns are handled
3. **Import timeouts**: Large file may take time to process
4. **Header selection issues**: Ensure correct header row is selected

### Performance Benchmarks
- **Generation time**: ~30-60 seconds
- **File size**: ~15-20 MB
- **Import time**: Target < 30 seconds
- **Memory usage**: Target < 500 MB

## Data Validation

### Sample Validation Queries
```sql
-- Check row count
SELECT COUNT(*) FROM test_large_complex;

-- Check null patterns
SELECT COUNT(*) FROM test_large_complex WHERE age IS NULL;
SELECT COUNT(*) FROM test_large_complex WHERE username = '';

-- Check data distributions
SELECT AVG(age), MIN(age), MAX(age) FROM test_large_complex;
SELECT AVG(income), MIN(income), MAX(income) FROM test_large_complex;

-- Check type inference
SELECT typeof(user_id), typeof(age), typeof(score), typeof(is_active) FROM test_large_complex LIMIT 1;
```

This comprehensive test dataset ensures that the CSV import functionality can handle real-world data scenarios with various data types, null patterns, and data quality issues. 