# Plot System Guide

The fresh application now includes a comprehensive plotting system that allows you to create visualizations from your query results.

## How to Use Plots

### 1. Open a Query Window
- Load a database/project in the application
- Use the sidebar to select a table
- This will open a query window with the table data

### 2. Create a Plot
- In the query window, click the "Add Plot" button
- This will automatically create a new plot window connected to your query

### 3. Configure the Plot
When the plot window opens, you'll see a collapsible "Plot Configuration" section that allows you to:

- **Select Plot Type**: Choose from:
  - Bar Chart
  - Box Plot  
  - Scatter Plot
  - Line Chart

- **Choose Columns**: Select which columns to use for:
  - X-axis (horizontal)
  - Y-axis (vertical)

- **Set Options**:
  - Show/Hide Legend
  - Show/Hide Grid
  - Set a custom title

The configuration panel can be collapsed/expanded by clicking the header, and changes are applied immediately.

### 4. Plot Types Explained

#### Bar Chart
- Groups data by X-axis values and sums Y-axis values
- Good for showing totals or counts by category
- Example: Sales by region, counts by category

#### Box Plot
- Shows statistical distribution of data
- Displays median, quartiles, and outliers
- Good for comparing distributions across groups
- Example: Salary distribution by department

#### Scatter Plot
- Shows individual data points
- Good for finding correlations or patterns
- Example: Height vs weight, temperature vs sales

#### Line Chart
- Connects data points in order
- Good for time series or sequential data
- Example: Sales over time, temperature trends

### 5. Data Requirements

For plots to work properly, your data should have:
- **Numeric columns** for X and Y axes
- **Valid numeric data** (no text in numeric columns)
- **Sufficient data points** (at least a few rows)

### 6. Real-time Updates

- When you modify the query in the source window, the plot will update automatically
- The plot window stays connected to its source query window
- You can have multiple plots from the same query window

### 7. Tips

- **Start with simple queries** to test the plotting
- **Use appropriate plot types** for your data:
  - Categorical data → Bar Chart
  - Continuous distributions → Box Plot
  - Correlations → Scatter Plot
  - Time series → Line Chart
- **Check your data types** - make sure X and Y columns contain numeric data
- **Experiment with different configurations** to find the best visualization

### Example Workflow

1. Load a database with sales data
2. Open the "sales" table query window
3. Click "Add Plot"
4. In the plot configuration:
   - Select "Bar Chart"
   - X Column: "region"
   - Y Column: "amount"
   - Title: "Sales by Region"
5. The plot will show total sales for each region
7. Modify the query to filter by date, and the plot updates automatically

## Technical Details

The plot system is built using:
- **egui_plot** for rendering
- **Real-time data binding** between query and plot windows
- **Configurable plot types** with different visualization algorithms
- **Automatic data validation** and error handling

The system is designed to be extensible - new plot types can be easily added by implementing the rendering methods in the `PlotWindow` struct. 