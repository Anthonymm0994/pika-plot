// Query and Data Transformation Module
class ArrowQuery {
    constructor() {
        this.currentData = null;
        this.filteredData = null;
        this.filters = new Map();
        this.derivedFields = new Map();
    }

    // Set current data
    setData(data) {
        this.currentData = data;
        this.filteredData = data;
        this.applyFilters();
    }

    // Get current filtered data
    getFilteredData() {
        return this.filteredData;
    }

    // Add filter for a field
    addFilter(fieldName, values) {
        if (values && values.length > 0) {
            this.filters.set(fieldName, values);
        } else {
            this.filters.delete(fieldName);
        }
        this.applyFilters();
    }

    // Remove filter for a field
    removeFilter(fieldName) {
        this.filters.delete(fieldName);
        this.applyFilters();
    }

    // Clear all filters
    clearFilters() {
        this.filters.clear();
        this.applyFilters();
    }

    // Apply all current filters
    applyFilters() {
        if (!this.currentData) {
            this.filteredData = null;
            return;
        }

        let filtered = this.currentData;

        // Apply each filter
        for (const [fieldName, values] of this.filters) {
            filtered = filtered.filter(d => values.includes(d[fieldName]));
        }

        // Apply derived fields
        if (this.derivedFields.size > 0) {
            filtered = this.applyDerivedFields(filtered);
        }

        this.filteredData = filtered;
    }

    // Add derived field
    addDerivedField(name, type, sourceField, options = {}) {
        this.derivedFields.set(name, {
            type,
            sourceField,
            options
        });
        this.applyFilters();
    }

    // Remove derived field
    removeDerivedField(name) {
        this.derivedFields.delete(name);
        this.applyFilters();
    }

    // Apply derived fields to data
    applyDerivedFields(data) {
        let result = data;

        for (const [fieldName, config] of this.derivedFields) {
            switch (config.type) {
                case 'difference':
                    result = this.addDifferenceField(result, fieldName, config.sourceField);
                    break;
                case 'binned':
                    result = this.addBinnedField(result, fieldName, config.sourceField, config.options.bins || 20);
                    break;
                case 'rolling_mean':
                    result = this.addRollingMeanField(result, fieldName, config.sourceField, config.options.window || 5);
                    break;
                case 'cumulative':
                    result = this.addCumulativeField(result, fieldName, config.sourceField);
                    break;
            }
        }

        return result;
    }

    // Add row difference field
    addDifferenceField(data, fieldName, sourceField) {
        try {
            return data.derive({
                [fieldName]: d => {
                    const values = data.get(sourceField).values();
                    const index = data.get(sourceField).indices().indexOf(d[sourceField]);
                    if (index > 0) {
                        return d[sourceField] - values[index - 1];
                    }
                    return null;
                }
            });
        } catch (error) {
            console.error('Error adding difference field:', error);
            return data;
        }
    }

    // Add binned field
    addBinnedField(data, fieldName, sourceField, bins) {
        try {
            const values = data.get(sourceField).values().filter(v => v != null);
            const min = Math.min(...values);
            const max = Math.max(...values);
            const binSize = (max - min) / bins;

            return data.derive({
                [fieldName]: d => {
                    if (d[sourceField] == null) return null;
                    const binIndex = Math.floor((d[sourceField] - min) / binSize);
                    return Math.min(binIndex, bins - 1);
                }
            });
        } catch (error) {
            console.error('Error adding binned field:', error);
            return data;
        }
    }

    // Add rolling mean field
    addRollingMeanField(data, fieldName, sourceField, window) {
        try {
            const values = data.get(sourceField).values();
            
            return data.derive({
                [fieldName]: d => {
                    const index = data.get(sourceField).indices().indexOf(d[sourceField]);
                    const start = Math.max(0, index - window + 1);
                    const windowValues = values.slice(start, index + 1).filter(v => v != null);
                    
                    if (windowValues.length === 0) return null;
                    return windowValues.reduce((a, b) => a + b, 0) / windowValues.length;
                }
            });
        } catch (error) {
            console.error('Error adding rolling mean field:', error);
            return data;
        }
    }

    // Add cumulative field
    addCumulativeField(data, fieldName, sourceField) {
        try {
            let cumulative = 0;
            
            return data.derive({
                [fieldName]: d => {
                    if (d[sourceField] != null) {
                        cumulative += d[sourceField];
                    }
                    return cumulative;
                }
            });
        } catch (error) {
            console.error('Error adding cumulative field:', error);
            return data;
        }
    }

    // Get aggregation statistics
    getAggregation(fieldName, aggregationType = 'count') {
        if (!this.filteredData) return null;

        try {
            switch (aggregationType) {
                case 'count':
                    return this.filteredData.numRows();
                case 'sum':
                    return this.filteredData.get(fieldName).sum();
                case 'mean':
                    return this.filteredData.get(fieldName).mean();
                case 'min':
                    return this.filteredData.get(fieldName).min();
                case 'max':
                    return this.filteredData.get(fieldName).max();
                case 'std':
                    return this.filteredData.get(fieldName).stdev();
                case 'unique':
                    return this.filteredData.get(fieldName).distinct().numRows();
                default:
                    return null;
            }
        } catch (error) {
            console.error('Error getting aggregation:', error);
            return null;
        }
    }

    // Get grouped aggregation
    getGroupedAggregation(groupField, valueField, aggregationType = 'count') {
        if (!this.filteredData) return null;

        try {
            let grouped = this.filteredData.groupby(groupField);
            
            switch (aggregationType) {
                case 'count':
                    return grouped.count();
                case 'sum':
                    return grouped.sum(valueField);
                case 'mean':
                    return grouped.mean(valueField);
                case 'min':
                    return grouped.min(valueField);
                case 'max':
                    return grouped.max(valueField);
                default:
                    return grouped.count();
            }
        } catch (error) {
            console.error('Error getting grouped aggregation:', error);
            return null;
        }
    }

    // Get data for plotting
    getPlotData(xField, yField, facetField = null) {
        if (!this.filteredData) return null;

        try {
            let plotData = this.filteredData.select([xField, yField, facetField].filter(Boolean));
            
            // Convert to array of objects for plotting
            return plotData.objects();
        } catch (error) {
            console.error('Error getting plot data:', error);
            return null;
        }
    }

    // Get sample data for preview
    getSampleData(limit = 100) {
        if (!this.filteredData) return null;

        try {
            return this.filteredData.slice(0, limit).objects();
        } catch (error) {
            console.error('Error getting sample data:', error);
            return null;
        }
    }

    // Get field statistics
    getFieldStatistics(fieldName) {
        if (!this.filteredData) return null;

        try {
            const column = this.filteredData.get(fieldName);
            if (!column) return null;

            const values = column.values().filter(v => v != null);
            
            if (values.length === 0) return null;

            const stats = {
                count: values.length,
                unique: new Set(values).size,
                nullCount: this.filteredData.numRows() - values.length
            };

            // Add numeric stats if applicable
            if (typeof values[0] === 'number') {
                stats.min = Math.min(...values);
                stats.max = Math.max(...values);
                stats.mean = values.reduce((a, b) => a + b, 0) / values.length;
                stats.median = this.calculateMedian(values);
            }

            return stats;
        } catch (error) {
            console.error('Error getting field statistics:', error);
            return null;
        }
    }

    // Calculate median
    calculateMedian(values) {
        const sorted = values.slice().sort((a, b) => a - b);
        const mid = Math.floor(sorted.length / 2);
        return sorted.length % 2 === 0
            ? (sorted[mid - 1] + sorted[mid]) / 2
            : sorted[mid];
    }

    // Get current filters
    getFilters() {
        return this.filters;
    }

    // Get derived fields
    getDerivedFields() {
        return this.derivedFields;
    }

    // Get data summary
    getDataSummary() {
        if (!this.filteredData) return null;

        return {
            totalRows: this.filteredData.numRows(),
            totalColumns: this.filteredData.numCols(),
            activeFilters: this.filters.size,
            derivedFields: this.derivedFields.size
        };
    }
}

// Export for use in other modules
window.ArrowQuery = ArrowQuery; 