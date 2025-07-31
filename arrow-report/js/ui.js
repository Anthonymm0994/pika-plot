// UI Module - Handles all user interactions and coordinates modules
class ArrowUI {
    constructor() {
        this.loader = new ArrowLoader();
        this.query = new ArrowQuery();
        this.plot = new ArrowPlot();
        this.currentData = null;
        this.schema = null;
        this.fileInfo = null;
        this.derivedFields = [];
    }

    init() {
        this.setupEventListeners();
        this.updateStatus('Ready to load Arrow files');
        this.plot.init('plotContainer');
    }

    setupEventListeners() {
        // File drop zone
        const dropZone = document.getElementById('fileDropZone');
        const fileInput = document.getElementById('fileInput');

        console.log('Setting up event listeners for:', {
            dropZone: dropZone ? 'found' : 'not found',
            fileInput: fileInput ? 'found' : 'not found'
        });

        // Drag and drop events
        dropZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.stopPropagation();
            dropZone.classList.add('drag-over');
            console.log('Drag over detected');
        });

        dropZone.addEventListener('dragleave', (e) => {
            e.preventDefault();
            e.stopPropagation();
            dropZone.classList.remove('drag-over');
            console.log('Drag leave detected');
        });

        dropZone.addEventListener('drop', (e) => {
            e.preventDefault();
            e.stopPropagation();
            dropZone.classList.remove('drag-over');
            const files = e.dataTransfer.files;
            console.log('Drop detected, files:', files.length);
            if (files.length > 0) {
                console.log('Processing dropped file:', files[0].name);
                this.handleFileSelect(files[0]);
            }
        });

        // File input change
        fileInput.addEventListener('change', (e) => {
            console.log('File input change detected, files:', e.target.files.length);
            if (e.target.files.length > 0) {
                console.log('Processing selected file:', e.target.files[0].name);
                this.handleFileSelect(e.target.files[0]);
            }
        });

        // Browse button
        const browseBtn = document.getElementById('browseBtn');
        if (browseBtn) {
            browseBtn.addEventListener('click', () => {
                console.log('Browse button clicked');
                fileInput.click();
            });
        } else {
            console.error('Browse button not found');
        }

        // Chart type selector
        document.getElementById('chartType').addEventListener('change', (e) => {
            this.onChartTypeChange(e.target.value);
        });

        // Filter controls
        document.getElementById('addFilterBtn').addEventListener('click', () => {
            this.addFilter();
        });

        // Derived fields
        document.getElementById('addDerivedFieldBtn').addEventListener('click', () => {
            this.addDerivedField();
        });

        // Export buttons
        document.getElementById('exportSvgBtn').addEventListener('click', () => {
            this.exportPlot('svg');
        });

        document.getElementById('exportPngBtn').addEventListener('click', () => {
            this.exportPlot('png');
        });

        // Reset button
        document.getElementById('resetBtn').addEventListener('click', () => {
            this.reset();
        });
    }

    async handleFileSelect(file) {
        console.log('File selected:', file.name, 'Size:', file.size, 'Type:', file.type);
        
        if (!this.loader.isValidArrowFile(file)) {
            this.showMessage('Please select a valid Arrow (.arrow) or Parquet (.parquet) file', 'error');
            return;
        }

        this.updateStatus('Loading file...');
        this.showMessage(`Loading ${file.name} (${this.formatFileSize(file.size)})...`, 'info');
        
        try {
            const result = await this.loader.loadArrowFile(file);
            this.onDataLoaded(result.table, result.schema, result.fileInfo);
        } catch (error) {
            console.error('Error loading file:', error);
            this.showMessage(`Error loading file: ${error.message}`, 'error');
            this.updateStatus('Error loading file');
        }
    }

    onDataLoaded(table, schema, fileInfo) {
        this.currentData = table;
        this.schema = schema;
        this.fileInfo = fileInfo;
        this.query.setData(table);

        this.updateFileInfo(fileInfo);
        this.updateFieldSelectors(schema);
        this.updateFilterControls();
        this.updateDerivedFieldsList();
        
        // Show relevant sections
        this.showSection('plotSection');
        this.showSection('derivedSection');
        this.showSection('exportSection');

        this.updateStatus(`Loaded ${fileInfo.size ? this.formatFileSize(fileInfo.size) : 'unknown size'} file with ${schema.numRows.toLocaleString()} rows and ${schema.numCols} columns`);
        this.showMessage('File loaded successfully!', 'success');
    }

    showSection(sectionId) {
        const section = document.getElementById(sectionId);
        if (section) {
            section.style.display = 'block';
        }
    }

    hideSection(sectionId) {
        const section = document.getElementById(sectionId);
        if (section) {
            section.style.display = 'none';
        }
    }

    updateFileInfo(fileInfo) {
        const fileInfoElement = document.getElementById('fileInfo');
        if (fileInfoElement) {
            fileInfoElement.style.display = 'block';
            fileInfoElement.innerHTML = `
                <h3>File Information</h3>
                <p><strong>Name:</strong> ${fileInfo.name}</p>
                <p><strong>Size:</strong> ${this.formatFileSize(fileInfo.size)}</p>
                <p><strong>Modified:</strong> ${new Date(fileInfo.lastModified).toLocaleString()}</p>
            `;
        }
    }

    updateFieldSelectors(schema) {
        const xFieldSelect = document.getElementById('xField');
        const yFieldSelect = document.getElementById('yField');
        const colorFieldSelect = document.getElementById('colorField');
        const sourceFieldSelect = document.getElementById('sourceField');

        // Clear existing options
        xFieldSelect.innerHTML = '<option value="">Select X field</option>';
        yFieldSelect.innerHTML = '<option value="">Select Y field</option>';
        colorFieldSelect.innerHTML = '<option value="">Select color field</option>';
        sourceFieldSelect.innerHTML = '<option value="">Select field...</option>';

        // Add field options with type information
        schema.fields.forEach(field => {
            const option = document.createElement('option');
            option.value = field.name;
            option.textContent = `${field.name} (${field.type})`;
            option.dataset.type = field.type;
            
            xFieldSelect.appendChild(option.cloneNode(true));
            yFieldSelect.appendChild(option.cloneNode(true));
            colorFieldSelect.appendChild(option.cloneNode(true));
            sourceFieldSelect.appendChild(option.cloneNode(true));
        });

        // Auto-select appropriate fields based on type
        this.autoSelectFields(schema);
    }

    autoSelectFields(schema) {
        const numericFields = schema.fields.filter(f => 
            f.type.includes('Int') || f.type.includes('Uint') || f.type.includes('Float')
        );
        const dateFields = schema.fields.filter(f => 
            f.type.includes('Date') || f.type.includes('Timestamp')
        );
        const categoricalFields = schema.fields.filter(f => 
            f.type.includes('Utf8') || f.type.includes('String')
        );

        // Auto-select X field (prefer date/time, then numeric)
        if (dateFields.length > 0) {
            document.getElementById('xField').value = dateFields[0].name;
        } else if (numericFields.length > 0) {
            document.getElementById('xField').value = numericFields[0].name;
        }

        // Auto-select Y field (prefer numeric)
        if (numericFields.length > 0) {
            const yField = numericFields.find(f => f.name !== document.getElementById('xField').value) || numericFields[0];
            document.getElementById('yField').value = yField.name;
        }

        // Auto-select color field (prefer categorical)
        if (categoricalFields.length > 0) {
            document.getElementById('colorField').value = categoricalFields[0].name;
        }
    }

    updateFilterControls() {
        const filterContainer = document.getElementById('filterContainer');
        filterContainer.innerHTML = '';

        if (!this.schema) return;

        this.schema.fields.forEach(field => {
            const filterGroup = document.createElement('div');
            filterGroup.className = 'filter-group';
            filterGroup.innerHTML = `
                <label>${field.name} (${field.type}):</label>
                <select class="filter-operator">
                    <option value="">No filter</option>
                    <option value="==">Equals</option>
                    <option value="!=">Not equals</option>
                    <option value=">">Greater than</option>
                    <option value="<">Less than</option>
                    <option value=">=">Greater than or equal</option>
                    <option value="<=">Less than or equal</option>
                    <option value="in">In list</option>
                    <option value="contains">Contains</option>
                </select>
                <input type="text" class="filter-value" placeholder="Value">
                <button class="remove-filter" onclick="this.parentElement.remove()">×</button>
            `;
            filterContainer.appendChild(filterGroup);
        });
    }

    onFilterChange() {
        this.updatePlot();
    }

    onChartTypeChange(chartType) {
        this.plot.setConfig({ chartType });
        this.updatePlot();
    }

    updatePlot() {
        if (!this.currentData) return;

        try {
            const xField = document.getElementById('xField').value;
            const yField = document.getElementById('yField').value;
            const colorField = document.getElementById('colorField').value;
            const chartType = document.getElementById('chartType').value;

            if (!xField || !yField) {
                this.plot.showEmptyState('Please select X and Y fields');
                return;
            }

            // Set the data in query module
            this.query.setData(this.currentData);
            
            // Get filtered data
            const filteredData = this.query.getFilteredData();
            
            // Apply derived fields
            const dataWithDerived = this.query.applyDerivedFields(filteredData);

            // Convert to array for plotting if it's an Arquero table
            let plotData = dataWithDerived;
            if (dataWithDerived && typeof dataWithDerived.objects === 'function') {
                plotData = dataWithDerived.objects();
            } else if (Array.isArray(dataWithDerived)) {
                plotData = dataWithDerived;
            }

            // Update plot
            this.plot.setConfig({
                chartType,
                xField,
                yField,
                colorField
            });
            this.plot.updatePlot(plotData);

        } catch (error) {
            console.error('Error updating plot:', error);
            this.plot.showErrorState(`Error updating plot: ${error.message}`);
        }
    }

    addFilter() {
        const filterContainer = document.getElementById('filterContainer');
        const filterGroup = document.createElement('div');
        filterGroup.className = 'filter-group';
        filterGroup.innerHTML = `
            <select class="filter-field">
                ${this.schema ? this.schema.fields.map(f => `<option value="${f.name}">${f.name} (${f.type})</option>`).join('') : ''}
            </select>
            <select class="filter-operator">
                <option value="==">Equals</option>
                <option value="!=">Not equals</option>
                <option value=">">Greater than</option>
                <option value="<">Less than</option>
                <option value=">=">Greater than or equal</option>
                <option value="<=">Less than or equal</option>
                <option value="in">In list</option>
                <option value="contains">Contains</option>
            </select>
            <input type="text" class="filter-value" placeholder="Value">
            <button class="remove-filter" onclick="this.parentElement.remove()">×</button>
        `;
        filterContainer.appendChild(filterGroup);
    }

    addDerivedField() {
        const derivedFieldType = document.getElementById('derivedFieldType').value;
        const sourceField = document.getElementById('sourceField').value;
        const outputField = document.getElementById('outputField').value;

        if (!sourceField || !outputField) {
            this.showMessage('Please select source field and enter output field name', 'error');
            return;
        }

        try {
            this.query.addDerivedField(derivedFieldType, sourceField, outputField);
            this.derivedFields.push({
                type: derivedFieldType,
                source: sourceField,
                output: outputField
            });
            this.updateDerivedFieldsList();
            this.updatePlot();
            this.showMessage('Derived field added successfully!', 'success');
        } catch (error) {
            this.showMessage(`Error adding derived field: ${error.message}`, 'error');
        }
    }

    updateDerivedFieldsList() {
        const derivedFieldsList = document.getElementById('derivedFieldsList');
        if (!derivedFieldsList) return;

        derivedFieldsList.innerHTML = '';
        this.derivedFields.forEach((field, index) => {
            const fieldElement = document.createElement('div');
            fieldElement.className = 'derived-field-item';
            fieldElement.innerHTML = `
                <span>${field.output} = ${field.type}(${field.source})</span>
                <button onclick="arrowUI.removeDerivedField(${index})">×</button>
            `;
            derivedFieldsList.appendChild(fieldElement);
        });
    }

    removeDerivedField(index) {
        this.derivedFields.splice(index, 1);
        this.updateDerivedFieldsList();
        this.updatePlot();
    }

    updateStatus(message) {
        const statusElement = document.getElementById('statusInfo');
        if (statusElement) {
            statusElement.textContent = message;
        }
    }

    showMessage(message, type = 'info') {
        const messageContainer = document.getElementById('messageContainer');
        if (!messageContainer) return;

        const messageElement = document.createElement('div');
        messageElement.className = `message ${type}`;
        messageElement.textContent = message;
        
        messageContainer.appendChild(messageElement);
        
        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (messageElement.parentNode) {
                messageElement.parentNode.removeChild(messageElement);
            }
        }, 5000);
    }

    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    exportPlot(format) {
        try {
            if (format === 'svg') {
                this.plot.exportAsSVG();
            } else if (format === 'png') {
                this.plot.exportAsPNG();
            }
            this.showMessage(`Plot exported as ${format.toUpperCase()}`, 'success');
        } catch (error) {
            this.showMessage(`Error exporting plot: ${error.message}`, 'error');
        }
    }

    reset() {
        this.currentData = null;
        this.schema = null;
        this.fileInfo = null;
        this.derivedFields = [];
        this.query.reset();
        this.plot.reset();
        
        // Reset UI
        document.getElementById('fileInfo').innerHTML = '';
        document.getElementById('fileInfo').style.display = 'none';
        document.getElementById('filterContainer').innerHTML = '';
        document.getElementById('derivedFieldsList').innerHTML = '';
        this.updateFieldSelectors({ fields: [] });
        this.updateStatus('Ready to load Arrow files');
        
        // Hide sections
        this.hideSection('plotSection');
        this.hideSection('derivedSection');
        this.hideSection('exportSection');
        
        this.showMessage('Application reset', 'info');
    }
}

// Initialize UI when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.arrowUI = new ArrowUI();
    window.arrowUI.init();
}); 