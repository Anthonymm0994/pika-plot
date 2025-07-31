// Arrow File Loader Module
class ArrowLoader {
    constructor() {
        this.currentData = null;
        this.schema = null;
        this.fileInfo = null;
    }

    /**
     * Load and parse an Arrow file
     * @param {File} file - The Arrow file to load
     * @returns {Promise<Object>} - Parsed data with schema and table
     */
    async loadArrowFile(file) {
        try {
            const arrayBuffer = await this.readFileAsArrayBuffer(file);
            const arrowData = await this.parseArrowData(arrayBuffer);
            const arqueroTable = this.convertToArqueroTable(arrowData);
            
            this.currentData = arqueroTable;
            this.schema = this.extractSchema(arrowData);
            this.fileInfo = {
                name: file.name,
                size: file.size,
                lastModified: file.lastModified
            };

            return {
                table: arqueroTable,
                schema: this.schema,
                fileInfo: this.fileInfo
            };
        } catch (error) {
            console.error('Error loading Arrow file:', error);
            throw error;
        }
    }

    /**
     * Read file as ArrayBuffer
     */
    readFileAsArrayBuffer(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result);
            reader.onerror = () => reject(reader.error);
            reader.readAsArrayBuffer(file);
        });
    }

    /**
     * Parse Arrow data using Apache Arrow JS
     */
    async parseArrowData(arrayBuffer) {
        try {
            // Check if Arrow library is available
            if (typeof Arrow === 'undefined') {
                // Try alternative global names
                if (typeof window.Arrow !== 'undefined') {
                    window.Arrow = window.Arrow;
                } else if (typeof globalThis.Arrow !== 'undefined') {
                    window.Arrow = globalThis.Arrow;
                } else {
                    throw new Error('Apache Arrow library not loaded. Please check your internet connection and ensure the script is loaded.');
                }
            }

            console.log('Arrow library loaded successfully');
            console.log('ArrayBuffer size:', arrayBuffer.byteLength);
            
            // Convert ArrayBuffer to Uint8Array
            const uint8Array = new Uint8Array(arrayBuffer);
            console.log('Uint8Array length:', uint8Array.length);
            console.log('First 32 bytes:', uint8Array.slice(0, 32));

            // Try different parsing approaches
            let table;
            
            try {
                // Method 1: Direct parsing
                table = Arrow.Table.from(uint8Array);
                console.log('Successfully parsed with Arrow.Table.from()');
            } catch (error1) {
                console.log('Method 1 failed:', error1.message);
                
                try {
                    // Method 2: Try with RecordBatchReader
                    const reader = Arrow.RecordBatchReader.from(uint8Array);
                    const batches = [];
                    for await (const batch of reader) {
                        batches.push(batch);
                    }
                    table = new Arrow.Table(batches);
                    console.log('Successfully parsed with RecordBatchReader');
                } catch (error2) {
                    console.log('Method 2 failed:', error2.message);
                    
                    try {
                        // Method 3: Try parsing as IPC format
                        const ipcReader = Arrow.RecordBatchReader.from(uint8Array);
                        const ipcBatches = [];
                        for await (const batch of ipcReader) {
                            ipcBatches.push(batch);
                        }
                        table = new Arrow.Table(ipcBatches);
                        console.log('Successfully parsed as IPC format');
                    } catch (error3) {
                        console.log('Method 3 failed:', error3.message);
                        throw new Error(`Failed to parse Arrow file. All parsing methods failed. Last error: ${error3.message}`);
                    }
                }
            }

            console.log('Parsed table info:', {
                numRows: table.numRows,
                numCols: table.numCols,
                schema: table.schema.fields.map(f => ({ name: f.name, type: f.type.toString() }))
            });

            return table;
        } catch (error) {
            console.error('Error parsing Arrow data:', error);
            if (error.message.includes('Arrow library not loaded')) {
                throw error;
            }
            throw new Error(`Failed to parse Arrow file. Please ensure it's a valid Arrow format. Error: ${error.message}`);
        }
    }

    /**
     * Convert Arrow table to Arquero table with enhanced data type handling
     */
    convertToArqueroTable(arrowTable) {
        const objects = this.arrowTableToObjects(arrowTable);
        return aq.from(objects);
    }

    /**
     * Convert Arrow table to JavaScript objects with enhanced type handling
     */
    arrowTableToObjects(arrowTable) {
        const objects = [];
        const numRows = arrowTable.numRows;
        const schema = arrowTable.schema;

        for (let i = 0; i < numRows; i++) {
            const row = {};
            
            for (let j = 0; j < schema.fields.length; j++) {
                const field = schema.fields[j];
                const column = arrowTable.getColumn(j);
                const value = column.get(i);
                
                // Enhanced type handling based on fresh project capabilities
                row[field.name] = this.convertValue(value, field.type);
            }
            
            objects.push(row);
        }

        return objects;
    }

    /**
     * Convert Arrow values to JavaScript values with enhanced type support
     */
    convertValue(value, arrowType) {
        if (value === null || value === undefined) {
            return null;
        }

        // Handle different Arrow data types
        const typeStr = arrowType.toString();
        switch (typeStr) {
            case 'Timestamp':
                return this.convertTimestamp(value, arrowType);
            case 'Date32':
            case 'Date64':
                return this.convertDate(value, arrowType);
            case 'Time32':
            case 'Time64':
                return this.convertTime(value, arrowType);
            case 'Int8':
            case 'Int16':
            case 'Int32':
            case 'Int64':
            case 'Uint8':
            case 'Uint16':
            case 'Uint32':
            case 'Uint64':
                return Number(value);
            case 'Float16':
            case 'Float32':
            case 'Float64':
                return Number(value);
            case 'Bool':
                return Boolean(value);
            case 'Utf8':
            case 'LargeUtf8':
                return String(value);
            default:
                return value;
        }
    }

    /**
     * Convert timestamp values with support for different units
     */
    convertTimestamp(value, arrowType) {
        if (value === null) return null;
        
        // Convert to milliseconds for JavaScript Date
        let milliseconds;
        const unit = arrowType.unit || 'MILLISECOND';
        switch (unit) {
            case 'SECOND':
                milliseconds = value * 1000;
                break;
            case 'MILLISECOND':
                milliseconds = value;
                break;
            case 'MICROSECOND':
                milliseconds = value / 1000;
                break;
            case 'NANOSECOND':
                milliseconds = value / 1000000;
                break;
            default:
                milliseconds = value;
        }
        
        return new Date(milliseconds);
    }

    /**
     * Convert date values
     */
    convertDate(value, arrowType) {
        if (value === null) return null;
        
        // Date32: days since epoch
        // Date64: milliseconds since epoch
        if (arrowType.toString() === 'Date32') {
            return new Date(value * 24 * 60 * 60 * 1000);
        } else {
            return new Date(value);
        }
    }

    /**
     * Convert time values to formatted strings
     */
    convertTime(value, arrowType) {
        if (value === null) return null;
        
        // Convert to seconds first
        let seconds;
        const unit = arrowType.unit || 'SECOND';
        switch (unit) {
            case 'SECOND':
                seconds = value;
                break;
            case 'MILLISECOND':
                seconds = value / 1000;
                break;
            case 'MICROSECOND':
                seconds = value / 1000000;
                break;
            case 'NANOSECOND':
                seconds = value / 1000000000;
                break;
            default:
                seconds = value;
        }
        
        // Format as HH:MM:SS
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = Math.floor(seconds % 60);
        
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }

    /**
     * Extract schema information with enhanced type details
     */
    extractSchema(arrowTable) {
        const schema = arrowTable.schema;
        const fields = [];
        
        for (let i = 0; i < schema.fields.length; i++) {
            const field = schema.fields[i];
            const column = arrowTable.getColumn(i);
            
            fields.push({
                name: field.name,
                type: field.type.toString(),
                nullable: field.nullable,
                metadata: field.metadata,
                // Add statistics for numeric columns
                stats: this.getColumnStats(column, field.type)
            });
        }
        
        return {
            fields: fields,
            numRows: arrowTable.numRows,
            numCols: arrowTable.numCols
        };
    }

    /**
     * Get column statistics for numeric and temporal columns
     */
    getColumnStats(column, arrowType) {
        const stats = {
            min: null,
            max: null,
            mean: null,
            nullCount: 0,
            uniqueCount: 0,
            type: arrowType.toString()
        };

        const values = [];
        const uniqueValues = new Set();

        for (let i = 0; i < column.length; i++) {
            const value = column.get(i);
            if (value === null || value === undefined) {
                stats.nullCount++;
                continue;
            }

            const convertedValue = this.convertValue(value, arrowType);
            values.push(convertedValue);
            uniqueValues.add(convertedValue);
        }

        stats.uniqueCount = uniqueValues.size;

        // Calculate statistics for numeric types
        if (this.isNumericType(arrowType)) {
            const numericValues = values.filter(v => typeof v === 'number' && !isNaN(v));
            if (numericValues.length > 0) {
                stats.min = Math.min(...numericValues);
                stats.max = Math.max(...numericValues);
                stats.mean = numericValues.reduce((a, b) => a + b, 0) / numericValues.length;
            }
        }

        return stats;
    }

    /**
     * Check if Arrow type is numeric
     */
    isNumericType(arrowType) {
        const typeStr = arrowType.toString();
        return typeStr.includes('Int') || typeStr.includes('Uint') || typeStr.includes('Float');
    }

    /**
     * Get field statistics for UI display
     */
    getFieldStats(fieldName) {
        if (!this.currentData) return null;
        
        const column = this.currentData.get(fieldName);
        if (!column) return null;

        const values = column.values();
        const nonNullValues = values.filter(v => v !== null && v !== undefined);
        
        return {
            count: values.length,
            nullCount: values.length - nonNullValues.length,
            uniqueCount: new Set(nonNullValues).size,
            min: nonNullValues.length > 0 ? Math.min(...nonNullValues) : null,
            max: nonNullValues.length > 0 ? Math.max(...nonNullValues) : null,
            mean: nonNullValues.length > 0 ? nonNullValues.reduce((a, b) => a + b, 0) / nonNullValues.length : null
        };
    }

    /**
     * Get unique values for a field
     */
    getUniqueValues(fieldName, limit = 100) {
        if (!this.currentData) return [];
        
        const column = this.currentData.get(fieldName);
        if (!column) return [];

        const uniqueValues = [...new Set(column.values())].filter(v => v !== null && v !== undefined);
        return uniqueValues.slice(0, limit);
    }

    /**
     * Validate if file is a valid Arrow file
     */
    isValidArrowFile(file) {
        return file.name.endsWith('.arrow') || file.name.endsWith('.parquet');
    }

    /**
     * Get current data
     */
    getCurrentData() {
        return this.currentData;
    }

    /**
     * Get current schema
     */
    getCurrentSchema() {
        return this.schema;
    }

    /**
     * Get file info
     */
    getFileInfo() {
        return this.fileInfo;
    }
}

// Export for use in other modules
window.ArrowLoader = ArrowLoader; 