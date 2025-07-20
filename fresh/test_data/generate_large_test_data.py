#!/usr/bin/env python3
"""
Large Complex Test Dataset Generator

This script generates a comprehensive test CSV with ~100,000 rows of realistic data
that mimics real-world patterns with various distributions and null patterns.

Features:
- Realistic user data with proper distributions
- Various null patterns (empty strings, "null", "NULL", "-", etc.)
- Mixed data types (integers, floats, booleans, dates, times, text)
- Garbage lines and empty rows for testing
- Different data quality scenarios

Usage:
    python generate_large_test_data.py

Output:
    test_large_complex.csv - ~100k line test dataset
"""

import csv
import random
import datetime
import numpy as np
from typing import List, Dict, Any

# Set random seed for reproducible results
random.seed(42)
np.random.seed(42)

# Data distributions and patterns
COUNTRIES = ['US', 'CA', 'UK', 'DE', 'FR', 'AU', 'JP', 'IT', 'BR', 'ES', 'NL', 'SE', 'NO', 'DK', 'FI']
DEVICE_TYPES = ['iphone', 'android', 'desktop', 'tablet', 'smartwatch']
OS_VERSIONS = {
    'iphone': ['iOS 16.0', 'iOS 16.1', 'iOS 16.2', 'iOS 16.3', 'iOS 16.4', 'iOS 16.5', 'iOS 16.6', 'iOS 17.0', 'iOS 17.1', 'iOS 17.2'],
    'android': ['Android 12', 'Android 13', 'Android 14'],
    'desktop': ['Windows 10', 'Windows 11', 'MacOS 13', 'MacOS 14', 'Ubuntu 22.04', 'Ubuntu 23.04'],
    'tablet': ['iPadOS 16', 'iPadOS 17', 'Android 13', 'Android 14'],
    'smartwatch': ['watchOS 9', 'watchOS 10', 'WearOS 4']
}
EMAIL_DOMAINS = ['gmail.com', 'outlook.com', 'yahoo.com', 'hotmail.com', 'icloud.com', 'protonmail.com']
REFERRAL_SOURCES = ['google', 'facebook', 'instagram', 'twitter', 'email', 'direct', 'organic', 'paid', 'referral']
PAYMENT_METHODS = ['credit_card', 'paypal', 'debit_card', 'apple_pay', 'google_pay', 'bank_transfer']
SUBSCRIPTION_TYPES = ['monthly', 'annual', 'quarterly', 'weekly']
PREMIUM_TIERS = ['free', 'standard', 'premium', 'enterprise']

def generate_realistic_age() -> int:
    """Generate age with realistic distribution (18-75, weighted towards 25-45)"""
    return int(np.random.normal(35, 12))
    
def generate_realistic_income() -> int:
    """Generate income with realistic distribution (20k-200k, right-skewed)"""
    # Log-normal distribution for income
    return int(np.random.lognormal(10.5, 0.5) * 1000)

def generate_boolean_with_null_probability(null_prob: float = 0.05) -> str:
    """Generate boolean with chance of null values"""
    if random.random() < null_prob:
        null_types = ['', 'null', 'NULL', '-', 'N/A']
        return random.choice(null_types)
    return random.choice(['true', 'false'])

def generate_float_with_null_probability(min_val: float, max_val: float, null_prob: float = 0.03) -> str:
    """Generate float with chance of null values"""
    if random.random() < null_prob:
        null_types = ['', 'null', 'NULL', '-', 'N/A']
        return random.choice(null_types)
    return f"{random.uniform(min_val, max_val):.2f}"

def generate_integer_with_null_probability(min_val: int, max_val: int, null_prob: float = 0.02) -> str:
    """Generate integer with chance of null values"""
    if random.random() < null_prob:
        null_types = ['', 'null', 'NULL', '-', 'N/A']
        return random.choice(null_types)
    return str(random.randint(min_val, max_val))

def generate_date_with_null_probability(start_date: datetime.date, end_date: datetime.date, null_prob: float = 0.01) -> str:
    """Generate date with chance of null values"""
    if random.random() < null_prob:
        null_types = ['', 'null', 'NULL', '-', 'N/A']
        return random.choice(null_types)
    
    days_between = (end_date - start_date).days
    random_days = random.randint(0, days_between)
    random_date = start_date + datetime.timedelta(days=random_days)
    return random_date.strftime('%Y-%m-%d')

def generate_time_with_null_probability(null_prob: float = 0.02) -> str:
    """Generate time with chance of null values"""
    if random.random() < null_prob:
        null_types = ['', 'null', 'NULL', '-', 'N/A']
        return random.choice(null_types)
    
    hour = random.randint(0, 23)
    minute = random.randint(0, 59)
    second = random.randint(0, 59)
    return f"{hour:02d}:{minute:02d}:{second:02d}"

def generate_datetime_with_null_probability(start_date: datetime.date, end_date: datetime.date, null_prob: float = 0.01) -> str:
    """Generate datetime with chance of null values"""
    if random.random() < null_prob:
        null_types = ['', 'null', 'NULL', '-', 'N/A']
        return random.choice(null_types)
    
    days_between = (end_date - start_date).days
    random_days = random.randint(0, days_between)
    random_date = start_date + datetime.timedelta(days=random_days)
    
    hour = random.randint(0, 23)
    minute = random.randint(0, 59)
    second = random.randint(0, 59)
    millisecond = random.randint(0, 999)
    
    return f"{random_date.strftime('%Y-%m-%d')}T{hour:02d}:{minute:02d}:{second:02d}.{millisecond:03d}Z"

def generate_realistic_score() -> str:
    """Generate score with realistic distribution (0-100, weighted towards 60-90)"""
    return generate_float_with_null_probability(0.0, 100.0, 0.02)

def generate_realistic_rating() -> str:
    """Generate rating with realistic distribution (1-5, weighted towards 3-5)"""
    return generate_float_with_null_probability(1.0, 5.0, 0.01)

def generate_realistic_balance() -> str:
    """Generate account balance with realistic distribution"""
    return generate_float_with_null_probability(0.0, 10000.0, 0.05)

def generate_realistic_spend() -> str:
    """Generate monthly spend with realistic distribution"""
    return generate_float_with_null_probability(0.0, 500.0, 0.03)

def generate_realistic_coordinates() -> tuple:
    """Generate realistic latitude/longitude coordinates"""
    # Major cities coordinates (simplified)
    cities = [
        (40.7128, -74.0060),  # NYC
        (34.0522, -118.2437), # LA
        (51.5074, -0.1278),   # London
        (48.8566, 2.3522),    # Paris
        (52.5200, 13.4050),   # Berlin
        (35.6762, 139.6503),  # Tokyo
        (41.9028, 12.4964),   # Rome
        (-33.8688, 151.2093), # Sydney
        (43.6532, -79.3832),  # Toronto
        (-23.5505, -46.6333), # Sao Paulo
    ]
    
    base_city = random.choice(cities)
    # Add some random variation
    lat = base_city[0] + random.uniform(-0.1, 0.1)
    lng = base_city[1] + random.uniform(-0.1, 0.1)
    return lat, lng

def generate_realistic_session_duration() -> str:
    """Generate session duration with realistic distribution (seconds)"""
    # Most sessions are 5-30 minutes, some longer
    minutes = np.random.exponential(15) + 5  # Exponential with minimum 5 minutes
    return str(int(minutes * 60))

def generate_realistic_page_views() -> str:
    """Generate page views with realistic distribution"""
    # Most users view 5-50 pages, some view many more
    views = np.random.exponential(20) + 5
    return str(int(views))

def generate_realistic_conversion_rate() -> str:
    """Generate conversion rate with realistic distribution (0-0.1)"""
    return generate_float_with_null_probability(0.0, 0.1, 0.15)

def generate_realistic_churn_probability() -> str:
    """Generate churn probability with realistic distribution (0-1)"""
    return generate_float_with_null_probability(0.0, 1.0, 0.10)

def generate_username() -> str:
    """Generate realistic username"""
    first_names = ['john', 'sarah', 'mike', 'anna', 'pierre', 'emma', 'taro', 'marco', 'maria', 'carlos']
    last_names = ['doe', 'smith', 'wilson', 'mueller', 'dupont', 'brown', 'yamamoto', 'rossi', 'silva', 'garcia']
    
    if random.random() < 0.8:  # 80% chance of normal username
        first = random.choice(first_names)
        last = random.choice(last_names)
        return f"{first}_{last}"
    else:  # 20% chance of null
        null_types = ['', 'null', 'NULL', '-', 'N/A']
        return random.choice(null_types)

def generate_timezone() -> str:
    """Generate realistic timezone"""
    timezones = ['UTC-8', 'UTC-7', 'UTC-6', 'UTC-5', 'UTC-4', 'UTC-3', 'UTC-2', 'UTC-1', 'UTC+0', 'UTC+1', 'UTC+2', 'UTC+3', 'UTC+4', 'UTC+5', 'UTC+6', 'UTC+7', 'UTC+8', 'UTC+9', 'UTC+10', 'UTC+11', 'UTC+12']
    return random.choice(timezones)

def generate_app_version() -> str:
    """Generate realistic app version"""
    major = random.randint(1, 3)
    minor = random.randint(0, 9)
    patch = random.randint(0, 9)
    return f"{major}.{minor}.{patch}"

def generate_row(user_id: int) -> List[str]:
    """Generate a single row of realistic data"""
    
    # Age with realistic distribution
    age = generate_realistic_age()
    age = max(18, min(75, age))  # Clamp to reasonable range
    
    # Income with realistic distribution
    income = generate_realistic_income()
    income = max(20000, min(200000, income))  # Clamp to reasonable range
    
    # Boolean fields with null probability
    is_active = generate_boolean_with_null_probability(0.02)
    has_verified_email = generate_boolean_with_null_probability(0.03)
    is_premium = generate_boolean_with_null_probability(0.02)
    
    # Dates
    start_date = datetime.date(2023, 1, 1)
    end_date = datetime.date(2023, 12, 31)
    registration_date = generate_date_with_null_probability(start_date, end_date, 0.01)
    join_date = registration_date  # Usually same as registration
    
    # Times
    last_login_time = generate_time_with_null_probability(0.02)
    
    # Datetimes
    created_at = generate_datetime_with_null_probability(start_date, end_date, 0.01)
    last_purchase_date = generate_datetime_with_null_probability(start_date, end_date, 0.05)
    
    # Categorical fields
    country_code = random.choice(COUNTRIES)
    premium_tier = random.choice(PREMIUM_TIERS)
    subscription_type = random.choice(SUBSCRIPTION_TYPES)
    payment_method = random.choice(PAYMENT_METHODS)
    email_domain = random.choice(EMAIL_DOMAINS)
    referral_source = random.choice(REFERRAL_SOURCES)
    device_type = random.choice(DEVICE_TYPES)
    
    # Device-specific OS version
    os_version = random.choice(OS_VERSIONS.get(device_type, ['Unknown']))
    
    # Numeric fields with realistic distributions
    score = generate_realistic_score()
    rating = generate_realistic_rating()
    login_count = generate_integer_with_null_probability(0, 500, 0.02)
    account_balance = generate_realistic_balance()
    monthly_spend = generate_realistic_spend()
    
    # Location data
    lat, lng = generate_realistic_coordinates()
    location_lat = f"{lat:.4f}"
    location_lng = f"{lng:.4f}"
    
    # Session and engagement data
    session_duration = generate_realistic_session_duration()
    page_views = generate_realistic_page_views()
    conversion_rate = generate_realistic_conversion_rate()
    churn_probability = generate_realistic_churn_probability()
    
    # Other fields
    username = generate_username()
    timezone = generate_timezone()
    app_version = generate_app_version()
    
    return [
        str(user_id),
        str(age),
        str(income),
        is_active,
        registration_date,
        last_login_time,
        premium_tier,
        country_code,
        score,
        rating,
        has_verified_email,
        username,
        created_at,
        login_count,
        subscription_type,
        payment_method,
        account_balance,
        monthly_spend,
        is_premium,
        email_domain,
        join_date,
        timezone,
        last_purchase_date,
        referral_source,
        device_type,
        os_version,
        app_version,
        location_lat,
        location_lng,
        session_duration,
        page_views,
        conversion_rate,
        churn_probability
    ]

def main():
    """Generate the large test dataset"""
    
    # Headers
    headers = [
        'user_id', 'age', 'income', 'is_active', 'registration_date', 'last_login_time',
        'premium_tier', 'country_code', 'score', 'rating', 'has_verified_email', 'username',
        'created_at', 'login_count', 'subscription_type', 'payment_method', 'account_balance',
        'monthly_spend', 'is_premium', 'email_domain', 'join_date', 'timezone',
        'last_purchase_date', 'referral_source', 'device_type', 'os_version', 'app_version',
        'location_lat', 'location_lng', 'session_duration', 'page_views', 'conversion_rate',
        'churn_probability'
    ]
    
    # File structure
    file_structure = [
        "# Large Complex Test Dataset",
        "# This file contains ~100,000 rows of realistic data with various distributions and null patterns",
        "# Used to test CSV import functionality, type inference, and data handling",
        "",
        "# Fake header line to test header row selection",
        "Fake Header Line - Ignore This",
        "",
        "# Another fake line",
        "Another fake line with some text",
        "",
        "# Real headers start here",
        ",".join(headers),
        "",
        "# Data starts here - 100,000 rows with realistic patterns"
    ]
    
    print("Generating large test dataset...")
    
    with open('test_large_complex.csv', 'w', newline='', encoding='utf-8') as csvfile:
        # Write file structure
        for line in file_structure:
            csvfile.write(line + '\n')
        
        # Write data rows
        writer = csv.writer(csvfile)
        
        # Generate 100,000 rows
        for i in range(1, 100001):
            if i % 10000 == 0:
                print(f"Generated {i:,} rows...")
            
            row = generate_row(i)
            writer.writerow(row)
            
            # Add some empty rows and garbage lines for testing
            if i % 5000 == 0:  # Every 5000 rows, add an empty row
                writer.writerow([''] * len(headers))
            
            if i % 10000 == 0:  # Every 10000 rows, add a garbage line
                garbage_lines = [
                    "This is a garbage line that should be ignored",
                    "ooOOoooo We end here.",
                    "Random text that doesn't match the pattern",
                    "ERROR: Invalid data format",
                    "DEBUG: Processing row data"
                ]
                writer.writerow([random.choice(garbage_lines)] + [''] * (len(headers) - 1))
    
    print(f"Generated test_large_complex.csv with ~100,000 rows")
    print("Dataset includes:")
    print("- Realistic user data with proper distributions")
    print("- Various null patterns (empty strings, 'null', 'NULL', '-', etc.)")
    print("- Mixed data types (integers, floats, booleans, dates, times, text)")
    print("- Garbage lines and empty rows for testing")
    print("- Different data quality scenarios")

if __name__ == "__main__":
    main() 