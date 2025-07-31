// Arrow File Loader Module
class ArrowLoader {
    constructor() {
        this.currentData = null;
        this.schema = null;
        this.fileInfo = null;
        this.onDataLoaded = null;
        this.onError = null;
    }

    // Set callbacks
    setCallbacks(onDataLoaded, onError) {
        this.onDataLoaded = onDataLoaded;
        this.onError = onError;
    }

    // Load Arrow file from File object
    async loadArrowFile(file) {
        try {
            this.updateStatus('Loading Arrow file...');
            
            // Read file as ArrayBuffer
            const arrayBuffer = await this.readFileAsArrayBuffer(file);
            
            // Parse Arrow data
            const arrowData = await this.parseArrowData(arrayBuffer);
            
            // Convert to Arquero table
            const table = await this.convertToArqueroTable(arrowData);
            
            // Extract schema information
            this.schema = this.extractSchema(arrowData);
            
            // Store file info
            this.fileInfo = {
                name: file.name,
                size: file.size,
                lastModified: file.lastModified,
                rowCount: table.numRows(),
                columnCount: table.numCols()
            };
            
            // Store current data
            this.currentData = table;
            
            this.updateStatus(`Loaded ${this.fileInfo.rowCount.toLocaleString()} rows, ${this.fileInfo.columnCount} columns`);
            
            // Notify success
            if (this.onDataLoaded) {
                this.onDataLoaded(this.currentData, this.schema, this.fileInfo);
            }
            
        } catch (error) {
            console.error('Error loading Arrow file:', error);
            this.updateStatus('Error loading file');
            
            if (this.onError) {
                this.onError(error);
            }
        }
    }

    // Read file as ArrayBuffer
    readFileAsArrayBuffer(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result);
            reader.onerror = () => reject(new Error('Failed to read file'));
            reader.readAsArrayBuffer(file);
        });
    }

    // Parse Arrow data using Apache Arrow JS
    async parseArrowData(arrayBuffer) {
        try {
            // Create Arrow table from ArrayBuffer
            const table = await arrow.Table.from(arrayBuffer);
            return table;
        } catch (error) {
            throw new Error(`Failed to parse Arrow data: ${error.message}`);
        }
    }

    // Convert Arrow table to Arquero table
    async convertToArqueroTable(arrowTable) {
        try {
            // Convert Arrow table to plain objects
            const objects = this.arrowTableToObjects(arrowTable);
            
            // Create Arquero table
            const table = aq.from(objects);
            return table;
        } catch (error) {
            throw new Error(`Failed to convert to Arquero table: ${error.message}`);
        }
    }

    // Convert Arrow table to array of objects
    arrowTableToObjects(arrowTable) {
        const objects = [];
        const numRows = arrowTable.numRows;
        const schema = arrowTable.schema;
        
        // Get column names
        const columnNames = schema.fields.map(field => field.name);
        
        // Convert each row
        for (let i = 0; i < numRows; i++) {
            const row = {};
            arrowTable.toArray().forEach((column, colIndex) => {
                const columnName = columnNames[colIndex];
                row[columnName] = column.get(i);
            });
            objects.push(row);
        }
        
        return objects;
    }

    // Extract schema information
    extractSchema(arrowTable) {
        const schema = {
            fields: [],
            fieldTypes: {},
            categoricalFields: [],
            numericFields: [],
            dateFields: []
        };
        
        arrowTable.schema.fields.forEach(field => {
            const fieldInfo = {
                name: field.name,
                type: field.type.toString(),
                nullable: field.nullable
            };
            
            schema.fields.push(fieldInfo);
            schema.fieldTypes[field.name] = field.type.toString();
            
            // Categorize fields
            const typeStr = field.type.toString().toLowerCase();
            if (typeStr.includes('string') || typeStr.includes('utf8') || typeStr.includes('largeutf8')) {
                schema.categoricalFields.push(field.name);
            } else if (typeStr.includes('int') || typeStr.includes('float') || typeStr.includes('double')) {
                schema.numericFields.push(field.name);
            } else if (typeStr.includes('date') || typeStr.includes('timestamp')) {
                schema.dateFields.push(field.name);
            }
        });
        
        return schema;
    }

    // Get current data
    getCurrentData() {
        return this.currentData;
    }

    // Get schema
    getSchema() {
        return this.schema;
    }

    // Get file info
    getFileInfo() {
        return this.fileInfo;
    }

    // Update status
    updateStatus(message) {
        const statusElement = document.getElementById('statusInfo');
        if (statusElement) {
            statusElement.textContent = message;
        }
    }

    // Get field statistics
    getFieldStats(fieldName) {
        if (!this.currentData) return null;
        
        try {
            const column = this.currentData.get(fieldName);
            if (!column) return null;
            
            const values = column.values();
            const nonNullValues = values.filter(v => v != null);
            
            const stats = {
                total: values.length,
                nonNull: nonNullValues.length,
                nullCount: values.length - nonNullValues.length,
                unique: new Set(nonNullValues).size
            };
            
            // Add numeric stats if applicable
            if (this.schema.numericFields.includes(fieldName)) {
                const numericValues = nonNullValues.filter(v => !isNaN(v));
                if (numericValues.length > 0) {
                    stats.min = Math.min(...numericValues);
                    stats.max = Math.max(...numericValues);
                    stats.mean = numericValues.reduce((a, b) => a + b, 0) / numericValues.length;
                }
            }
            
            return stats;
        } catch (error) {
            console.error('Error getting field stats:', error);
            return null;
        }
    }

    // Get unique values for categorical field
    getUniqueValues(fieldName, limit = 100) {
        if (!this.currentData) return [];
        
        try {
            const column = this.currentData.get(fieldName);
            if (!column) return [];
            
            const values = column.values();
            const uniqueValues = [...new Set(values.filter(v => v != null))];
            
            // Sort and limit
            return uniqueValues.sort().slice(0, limit);
        } catch (error) {
            console.error('Error getting unique values:', error);
            return [];
        }
    }

    // Validate file type
    isValidArrowFile(file) {
        const validExtensions = ['.arrow', '.parquet'];
        const fileName = file.name.toLowerCase();
        return validExtensions.some(ext => fileName.endsWith(ext));
    }
}

// Export for use in other modules
window.ArrowLoader = ArrowLoader; 