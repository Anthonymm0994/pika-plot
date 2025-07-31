#!/usr/bin/env python3
"""
Generate large test dataset for duplicate detection testing.
This script creates a complex CSV file with various duplicate patterns.
"""

import csv
import random
import datetime
from typing import List, Dict, Any

# Configuration
NUM_RECORDS = 10000
NUM_GROUPS = 500
DUPLICATE_RATIO = 0.3  # 30% of records should be duplicates

# Data templates
DEPARTMENTS = ['Engineering', 'Marketing', 'Sales', 'HR', 'Finance', 'Operations', 'Legal', 'IT']
NAMES = [
    'John', 'Jane', 'Bob', 'Alice', 'Charlie', 'David', 'Emma', 'Frank', 'Grace', 'Henry',
    'Ivy', 'Jack', 'Kate', 'Liam', 'Mia', 'Noah', 'Olivia', 'Paul', 'Quinn', 'Ruby',
    'Sam', 'Tina', 'Uma', 'Victor', 'Wendy', 'Xander', 'Yara', 'Zoe', 'Alex', 'Blake',
    'Casey', 'Drew', 'Eden', 'Finley', 'Gray', 'Harper', 'Indigo', 'Jordan', 'Kai', 'Luna'
]

def generate_record(group_id: str, record_id: int, is_duplicate: bool = False) -> Dict[str, Any]:
    """Generate a single record with realistic data."""
    
    # Base data
    name = random.choice(NAMES)
    age = random.randint(22, 65)
    salary = random.randint(30000, 150000)
    is_active = random.choice([True, False])
    department = random.choice(DEPARTMENTS)
    rating = round(random.uniform(1.0, 5.0), 1)
    
    # Generate date within last 2 years
    days_ago = random.randint(0, 730)
    created_date = datetime.datetime.now() - datetime.timedelta(days=days_ago)
    created_date_str = created_date.strftime('%Y-%m-%d')
    
    # Add some null values randomly
    if random.random() < 0.1:  # 10% chance of null age
        age = None
    if random.random() < 0.05:  # 5% chance of null salary
        salary = None
    if random.random() < 0.08:  # 8% chance of null rating
        rating = None
    if random.random() < 0.03:  # 3% chance of null department
        department = None
    
    return {
        'group_id': group_id,
        'name': name,
        'age': age,
        'salary': salary,
        'is_active': is_active,
        'created_date': created_date_str,
        'department': department,
        'rating': rating,
        'record_id': record_id
    }

def create_duplicate_records(base_record: Dict[str, Any], num_duplicates: int) -> List[Dict[str, Any]]:
    """Create duplicate records based on a base record."""
    duplicates = []
    for i in range(num_duplicates):
        duplicate = base_record.copy()
        duplicate['record_id'] = f"{base_record['record_id']}_dup_{i+1}"
        duplicates.append(duplicate)
    return duplicates

def generate_large_dataset():
    """Generate a large dataset with controlled duplicate patterns."""
    
    records = []
    record_id = 1
    
    # Generate groups
    for group_num in range(NUM_GROUPS):
        group_id = f"GROUP_{group_num:03d}"
        
        # Determine how many records in this group
        group_size = random.randint(1, 50)
        
        # Create base records for this group
        group_records = []
        for i in range(group_size):
            base_record = generate_record(group_id, record_id)
            group_records.append(base_record)
            record_id += 1
        
        # Add duplicates to some groups
        if random.random() < DUPLICATE_RATIO:
            # Select a random record to duplicate
            base_record = random.choice(group_records)
            num_duplicates = random.randint(1, 5)
            duplicates = create_duplicate_records(base_record, num_duplicates)
            group_records.extend(duplicates)
            record_id += num_duplicates
        
        records.extend(group_records)
    
    # Shuffle records to make it more realistic
    random.shuffle(records)
    
    return records

def write_csv(records: List[Dict[str, Any]], filename: str):
    """Write records to CSV file."""
    if not records:
        return
    
    fieldnames = records[0].keys()
    
    with open(filename, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(records)

def main():
    """Main function to generate the large dataset."""
    print("Generating large test dataset...")
    
    # Set random seed for reproducibility
    random.seed(42)
    
    # Generate records
    records = generate_large_dataset()
    
    # Write to CSV
    output_file = 'large_complex_dataset.csv'
    write_csv(records, output_file)
    
    print(f"Generated {len(records)} records in {output_file}")
    print(f"Number of unique groups: {len(set(r['group_id'] for r in records))}")
    
    # Analyze duplicates
    group_counts = {}
    for record in records:
        group_id = record['group_id']
        group_counts[group_id] = group_counts.get(group_id, 0) + 1
    
    duplicate_groups = [g for g, count in group_counts.items() if count > 1]
    print(f"Groups with duplicates: {len(duplicate_groups)}")
    
    # Show some statistics
    total_duplicates = sum(count - 1 for count in group_counts.values() if count > 1)
    print(f"Total duplicate records: {total_duplicates}")
    print(f"Duplicate ratio: {total_duplicates / len(records):.2%}")

if __name__ == "__main__":
    main() 