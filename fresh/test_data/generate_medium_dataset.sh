#!/bin/bash

# Generate medium test dataset with ~10k rows
# This script extends the existing test_medium_complex.csv with more realistic data

echo "Generating medium test dataset..."

# Start with the existing 50 rows
cp test_medium_complex.csv test_medium_complex_full.csv

# Generate additional rows (50 to 10,000)
for i in {51..10000}; do
    # Generate realistic data patterns
    age=$((18 + RANDOM % 58))  # 18-75
    income=$((20000 + RANDOM % 180000))  # 20k-200k
    
    # Boolean with null probability
    if [ $((RANDOM % 100)) -lt 3 ]; then
        is_active=""
    else
        is_active=$([ $((RANDOM % 2)) -eq 0 ] && echo "true" || echo "false")
    fi
    
    # Float values
    score=$(echo "scale=1; $((60 + RANDOM % 40)).$((RANDOM % 10))" | bc)
    rating=$(echo "scale=1; $((30 + RANDOM % 20)).$((RANDOM % 10))" | bc)
    
    # Random selection from arrays
    countries=("US" "CA" "UK" "DE" "FR" "AU" "JP" "IT" "BR" "ES" "NL" "SE" "NO" "DK" "FI")
    country_code=${countries[$((RANDOM % ${#countries[@]}))]}
    
    tiers=("free" "standard" "premium" "enterprise")
    premium_tier=${tiers[$((RANDOM % ${#tiers[@]}))]}
    
    devices=("iphone" "android" "desktop" "tablet" "smartwatch")
    device_type=${devices[$((RANDOM % ${#devices[@]}))]}
    
    # Generate date (2023)
    month=$((1 + RANDOM % 12))
    day=$((1 + RANDOM % 28))
    registration_date="2023-$(printf "%02d" $month)-$(printf "%02d" $day)"
    
    # Generate time
    hour=$((RANDOM % 24))
    minute=$((RANDOM % 60))
    second=$((RANDOM % 60))
    last_login_time="$(printf "%02d" $hour):$(printf "%02d" $minute):$(printf "%02d" $second)"
    
    # Generate coordinates (simplified)
    lat=$(echo "scale=4; $((RANDOM % 180 - 90)).$((RANDOM % 10000))" | bc)
    lng=$(echo "scale=4; $((RANDOM % 360 - 180)).$((RANDOM % 10000))" | bc)
    
    # Generate session data
    session_duration=$((300 + RANDOM % 3300))  # 5-60 minutes
    page_views=$((5 + RANDOM % 195))  # 5-200
    
    # Generate rates
    conversion_rate=$(echo "scale=3; $((RANDOM % 100)).$((RANDOM % 1000))" | bc)
    churn_probability=$(echo "scale=2; $((RANDOM % 100)).$((RANDOM % 100))" | bc)
    
    # Generate account data
    account_balance=$(echo "scale=2; $((RANDOM % 10000)).$((RANDOM % 100))" | bc)
    monthly_spend=$(echo "scale=2; $((RANDOM % 500)).$((RANDOM % 100))" | bc)
    
    # Generate login count
    login_count=$((RANDOM % 500))
    
    # Generate username
    first_names=("john" "sarah" "mike" "anna" "pierre" "emma" "taro" "marco" "maria" "carlos")
    last_names=("doe" "smith" "wilson" "mueller" "dupont" "brown" "yamamoto" "rossi" "silva" "garcia")
    first=${first_names[$((RANDOM % ${#first_names[@]}))]}
    last=${last_names[$((RANDOM % ${#last_names[@]}))]}
    username="${first}_${last}_${i}"
    
    # Generate email domain
    domains=("gmail.com" "outlook.com" "yahoo.com" "hotmail.com" "icloud.com")
    email_domain=${domains[$((RANDOM % ${#domains[@]}))]}
    
    # Generate timezone
    timezones=("UTC-8" "UTC-7" "UTC-6" "UTC-5" "UTC-4" "UTC-3" "UTC-2" "UTC-1" "UTC+0" "UTC+1" "UTC+2" "UTC+3" "UTC+4" "UTC+5" "UTC+6" "UTC+7" "UTC+8" "UTC+9" "UTC+10" "UTC+11" "UTC+12")
    timezone=${timezones[$((RANDOM % ${#timezones[@]}))]}
    
    # Generate app version
    major=$((1 + RANDOM % 3))
    minor=$((RANDOM % 10))
    patch=$((RANDOM % 10))
    app_version="${major}.${minor}.${patch}"
    
    # Generate OS version based on device
    case $device_type in
        "iphone")
            os_versions=("iOS 16.0" "iOS 16.1" "iOS 16.2" "iOS 16.3" "iOS 16.4" "iOS 16.5" "iOS 16.6" "iOS 17.0" "iOS 17.1" "iOS 17.2")
            ;;
        "android")
            os_versions=("Android 12" "Android 13" "Android 14")
            ;;
        "desktop")
            os_versions=("Windows 10" "Windows 11" "MacOS 13" "MacOS 14" "Ubuntu 22.04" "Ubuntu 23.04")
            ;;
        "tablet")
            os_versions=("iPadOS 16" "iPadOS 17" "Android 13" "Android 14")
            ;;
        "smartwatch")
            os_versions=("watchOS 9" "watchOS 10" "WearOS 4")
            ;;
        *)
            os_versions=("Unknown")
            ;;
    esac
    os_version=${os_versions[$((RANDOM % ${#os_versions[@]}))]}
    
    # Generate other fields
    subscription_types=("monthly" "annual" "quarterly" "weekly")
    subscription_type=${subscription_types[$((RANDOM % ${#subscription_types[@]}))]}
    
    payment_methods=("credit_card" "paypal" "debit_card" "apple_pay" "google_pay" "bank_transfer")
    payment_method=${payment_methods[$((RANDOM % ${#payment_methods[@]}))]}
    
    referral_sources=("google" "facebook" "instagram" "twitter" "email" "direct" "organic" "paid" "referral")
    referral_source=${referral_sources[$((RANDOM % ${#referral_sources[@]}))]}
    
    # Generate boolean fields
    if [ $((RANDOM % 100)) -lt 3 ]; then
        has_verified_email=""
    else
        has_verified_email=$([ $((RANDOM % 2)) -eq 0 ] && echo "true" || echo "false")
    fi
    
    if [ $((RANDOM % 100)) -lt 2 ]; then
        is_premium=""
    else
        is_premium=$([ $((RANDOM % 2)) -eq 0 ] && echo "true" || echo "false")
    fi
    
    # Generate dates
    created_at="${registration_date}T${last_login_time}.$((RANDOM % 1000))Z"
    join_date="$registration_date"
    last_purchase_date="${registration_date}T$((10 + RANDOM % 14)):$((RANDOM % 60)):$((RANDOM % 60)).$((RANDOM % 1000))Z"
    
    # Write the row
    echo "$i,$age,$income,$is_active,$registration_date,$last_login_time,$premium_tier,$country_code,$score,$rating,$has_verified_email,$username,$created_at,$login_count,$subscription_type,$payment_method,$account_balance,$monthly_spend,$is_premium,$email_domain,$join_date,$timezone,$last_purchase_date,$referral_source,$device_type,$os_version,$app_version,$location_lat,$location_lng,$session_duration,$page_views,$conversion_rate,$churn_probability" >> test_medium_complex_full.csv
    
    # Add some empty rows and garbage lines for testing
    if [ $((i % 500)) -eq 0 ]; then
        echo ",,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,," >> test_medium_complex_full.csv
    fi
    
    if [ $((i % 1000)) -eq 0 ]; then
        garbage_lines=("This is a garbage line that should be ignored" "ooOOoooo We end here." "Random text that doesn't match the pattern" "ERROR: Invalid data format" "DEBUG: Processing row data")
        garbage_line=${garbage_lines[$((RANDOM % ${#garbage_lines[@]}))]}
        echo "$garbage_line,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,," >> test_medium_complex_full.csv
    fi
    
    # Progress indicator
    if [ $((i % 1000)) -eq 0 ]; then
        echo "Generated $i rows..."
    fi
done

echo "Generated test_medium_complex_full.csv with ~10,000 rows"
echo "Dataset includes realistic data patterns, null values, and garbage lines for comprehensive testing." 