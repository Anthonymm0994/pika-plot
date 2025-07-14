# BAR Plot Documentation

**Title**: Average Sales by Category
**Type**: bar
**Query**: SELECT category, AVG(sales) as avg_sales FROM plot_test_data GROUP BY category
**X Column**: category
**Y Column**: avg_sales

## Visual Requirements

### Axes and Labels
- X-axis labeled with 'category' and appropriate units
- Y-axis labeled with 'avg_sales' and appropriate units
- Tick marks clearly visible and properly spaced
- Numbers readable and appropriately formatted

### Legend
- Positioned in top-right corner (default)
- Clear symbols corresponding to data series
- Readable text with proper font sizing
- Background contrast for visibility

### Grid and Layout
- Grid lines present for better readability
- Adequate margins for labels and legends
- Professional color scheme
- Appropriate plot area sizing

### Interactive Features
- Zoom capability with mouse wheel
- Pan functionality with mouse drag
- Tooltips showing data values on hover
- Legend toggle for series visibility
- Crosshairs for precise data targeting

## Export Quality
- PNG: High resolution (300 DPI minimum)
- SVG: Scalable vector format
- Proper font embedding
- Consistent styling across formats
