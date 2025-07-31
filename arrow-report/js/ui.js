// UI Module - Handles all user interactions and coordinates modules
class ArrowUI {
    constructor() {
        this.loader = new ArrowLoader();
        this.query = new ArrowQuery();
        this.plot = new ArrowPlot();
        
        this.currentSchema = null;
        this.currentFileInfo = null;
        
        this.init();
    }

    // Initialize UI
    init() {
        this.setupEventListeners();
        this.plot.init('plotContainer');
        
        // Set loader callbacks
        this.loader.setCallbacks(
            (data, schema, fileInfo) => this.onDataLoaded(data, schema, fileInfo),
            (error) => this.onError(error)
        );
    }

    // Setup event listeners
    setupEventListeners() {
        // File loading
        const dropZone = document.getElementById('dropZone');
        const fileInput = document.getElementById('fileInput');
        const filePicker = document.getElementById('filePicker');

        // Drag and drop
        dropZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            dropZone.classList.add('dragover');
        });

        dropZone.addEventListener('dragleave', () => {
            dropZone.classList.remove('dragover');
        });

        dropZone.addEventListener('drop', (e) => {
            e.preventDefault();
            dropZone.classList.remove('dragover');
            
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.handleFileSelect(files[0]);
            }
        });

        // File picker
        filePicker.addEventListener('click', () => {
            fileInput.click();
        });

        fileInput.addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                this.handleFileSelect(e.target.files[0]);
            }
        });

        // Plot controls
        const updatePlotBtn = document.getElementById('updatePlot');
        updatePlotBtn.addEventListener('click', () => this.updatePlot());

        // Chart type change
        const chartTypeSelect = document.getElementById('chartType');
        chartTypeSelect.addEventListener('change', () => this.onChartTypeChange());

        // Derived fields
        const addDerivedBtn = document.getElementById('addDerived');
        addDerivedBtn.addEventListener('click', () => this.addDerivedField());

        // Window resize
        window.addEventListener('resize', () => {
            this.plot.resize();
        });
    }

    // Handle file selection
    async handleFileSelect(file) {
        if (!this.loader.isValidArrowFile(file)) {
            this.showError('Please select a valid Arrow or Parquet file (.arrow, .parquet)');
            return;
        }

        try {
            await this.loader.loadArrowFile(file);
        } catch (error) {
            this.showError(`Error loading file: ${error.message}`);
        }
    }

    // Called when data is successfully loaded
    onDataLoaded(data, schema, fileInfo) {
        this.currentSchema = schema;
        this.currentFileInfo = fileInfo;
        
        // Set data in query module
        this.query.setData(data);
        
        // Update UI
        this.updateFileInfo(fileInfo);
        this.updateFieldSelectors(schema);
        this.showSections();
        this.updateStatus();
        
        // Show success message
        this.showSuccess(`Successfully loaded ${fileInfo.rowCount.toLocaleString()} rows`);
    }

    // Called when an error occurs
    onError(error) {
        this.showError(`Error: ${error.message}`);
        this.updateStatus('Error loading file');
    }

    // Update file info display
    updateFileInfo(fileInfo) {
        const fileInfoDiv = document.getElementById('fileInfo');
        const fileStatsDiv = document.getElementById('fileStats');
        
        fileStatsDiv.innerHTML = `
            <div><strong>Name:</strong> ${fileInfo.name}</div>
            <div><strong>Size:</strong> ${this.formatFileSize(fileInfo.size)}</div>
            <div><strong>Rows:</strong> ${fileInfo.rowCount.toLocaleString()}</div>
            <div><strong>Columns:</strong> ${fileInfo.columnCount}</div>
            <div><strong>Modified:</strong> ${new Date(fileInfo.lastModified).toLocaleString()}</div>
        `;
        
        fileInfoDiv.style.display = 'block';
    }

    // Update field selectors
    updateFieldSelectors(schema) {
        const xAxisSelect = document.getElementById('xAxis');
        const yAxisSelect = document.getElementById('yAxis');
        const facetBySelect = document.getElementById('facetBy');
        const derivedSourceSelect = document.getElementById('derivedSource');
        
        // Clear existing options
        xAxisSelect.innerHTML = '<option value="">Select field...</option>';
        yAxisSelect.innerHTML = '<option value="">Select field...</option>';
        facetBySelect.innerHTML = '<option value="">No faceting</option>';
        derivedSourceSelect.innerHTML = '<option value="">Select field...</option>';
        
        // Add all fields to X and Y axis
        schema.fields.forEach(field => {
            const option = document.createElement('option');
            option.value = field.name;
            option.textContent = field.name;
            
            xAxisSelect.appendChild(option.cloneNode(true));
            yAxisSelect.appendChild(option.cloneNode(true));
            derivedSourceSelect.appendChild(option.cloneNode(true));
        });
        
        // Add categorical fields to facet
        schema.categoricalFields.forEach(field => {
            const option = document.createElement('option');
            option.value = field;
            option.textContent = field;
            facetBySelect.appendChild(option);
        });
        
        // Add numeric fields to derived source
        schema.numericFields.forEach(field => {
            const option = document.createElement('option');
            option.value = field;
            option.textContent = field;
            derivedSourceSelect.appendChild(option);
        });
    }

    // Show/hide sections based on data availability
    showSections() {
        const sections = ['filterSection', 'plotSection', 'derivedSection'];
        sections.forEach(sectionId => {
            const section = document.getElementById(sectionId);
            if (section) {
                section.style.display = 'block';
            }
        });
        
        // Update filter controls
        this.updateFilterControls();
    }

    // Update filter controls
    updateFilterControls() {
        const filterControls = document.getElementById('filterControls');
        filterControls.innerHTML = '';
        
        if (!this.currentSchema) return;
        
        this.currentSchema.categoricalFields.forEach(field => {
            const filterGroup = this.createFilterGroup(field);
            filterControls.appendChild(filterGroup);
        });
    }

    // Create filter group for a field
    createFilterGroup(fieldName) {
        const filterGroup = document.createElement('div');
        filterGroup.className = 'filter-group';
        
        const uniqueValues = this.loader.getUniqueValues(fieldName, 50);
        
        filterGroup.innerHTML = `
            <label>${fieldName}</label>
            <div class="filter-values">
                ${uniqueValues.map(value => `
                    <label class="filter-checkbox">
                        <input type="checkbox" value="${value}" data-field="${fieldName}" checked>
                        ${value}
                    </label>
                `).join('')}
            </div>
        `;
        
        // Add event listeners
        const checkboxes = filterGroup.querySelectorAll('input[type="checkbox"]');
        checkboxes.forEach(checkbox => {
            checkbox.addEventListener('change', () => this.onFilterChange(fieldName));
        });
        
        return filterGroup;
    }

    // Handle filter change
    onFilterChange(fieldName) {
        const checkboxes = document.querySelectorAll(`input[data-field="${fieldName}"]:checked`);
        const selectedValues = Array.from(checkboxes).map(cb => cb.value);
        
        this.query.addFilter(fieldName, selectedValues);
        this.updatePlot();
        this.updateStatus();
    }

    // Handle chart type change
    onChartTypeChange() {
        const chartType = document.getElementById('chartType').value;
        const yAxisSelect = document.getElementById('yAxis');
        
        // Show/hide Y axis based on chart type
        if (chartType === 'histogram') {
            yAxisSelect.parentElement.style.display = 'none';
        } else {
            yAxisSelect.parentElement.style.display = 'block';
        }
    }

    // Update plot
    updatePlot() {
        const chartType = document.getElementById('chartType').value;
        const xField = document.getElementById('xAxis').value;
        const yField = document.getElementById('yAxis').value;
        const facetField = document.getElementById('facetBy').value;
        const bins = parseInt(document.getElementById('bins').value) || 20;
        
        if (!xField) {
            this.plot.showEmptyState();
            return;
        }
        
        // Get plot data
        const plotData = this.query.getPlotData(xField, yField, facetField);
        
        // Set plot configuration
        this.plot.setConfig({
            chartType,
            xField,
            yField,
            facetField: facetField || null,
            bins
        });
        
        // Update plot
        this.plot.updatePlot(plotData);
        
        // Update status
        this.updateStatus();
    }

    // Add derived field
    addDerivedField() {
        const type = document.getElementById('derivedType').value;
        const sourceField = document.getElementById('derivedSource').value;
        const fieldName = document.getElementById('derivedName').value;
        
        if (!sourceField || !fieldName) {
            this.showError('Please select a source field and provide a field name');
            return;
        }
        
        // Add derived field
        this.query.addDerivedField(fieldName, type, sourceField, {
            bins: parseInt(document.getElementById('bins').value) || 20
        });
        
        // Update derived fields list
        this.updateDerivedFieldsList();
        
        // Update plot
        this.updatePlot();
        
        // Clear form
        document.getElementById('derivedName').value = '';
        
        this.showSuccess(`Added derived field: ${fieldName}`);
    }

    // Update derived fields list
    updateDerivedFieldsList() {
        const derivedFieldsList = document.getElementById('derivedFieldsList');
        const derivedFields = this.query.getDerivedFields();
        
        derivedFieldsList.innerHTML = '';
        
        derivedFields.forEach((config, fieldName) => {
            const fieldItem = document.createElement('div');
            fieldItem.className = 'derived-field-item';
            
            fieldItem.innerHTML = `
                <div class="derived-field-info">
                    <div class="derived-field-name">${fieldName}</div>
                    <div class="derived-field-details">
                        Type: ${config.type}, Source: ${config.sourceField}
                    </div>
                </div>
                <button class="btn btn-danger btn-sm" onclick="arrowUI.removeDerivedField('${fieldName}')">
                    Remove
                </button>
            `;
            
            derivedFieldsList.appendChild(fieldItem);
        });
    }

    // Remove derived field
    removeDerivedField(fieldName) {
        this.query.removeDerivedField(fieldName);
        this.updateDerivedFieldsList();
        this.updatePlot();
        this.showSuccess(`Removed derived field: ${fieldName}`);
    }

    // Update status
    updateStatus() {
        const summary = this.query.getDataSummary();
        if (summary) {
            const statusStats = document.getElementById('statusStats');
            statusStats.innerHTML = `
                <span>Rows: ${summary.totalRows.toLocaleString()}</span>
                <span>Filters: ${summary.activeFilters}</span>
                <span>Derived: ${summary.derivedFields}</span>
            `;
        }
    }

    // Show success message
    showSuccess(message) {
        this.showMessage(message, 'success');
    }

    // Show error message
    showError(message) {
        this.showMessage(message, 'error');
    }

    // Show message
    showMessage(message, type) {
        // Remove existing messages
        const existingMessages = document.querySelectorAll('.message');
        existingMessages.forEach(msg => msg.remove());
        
        // Create new message
        const messageDiv = document.createElement('div');
        messageDiv.className = `message ${type}`;
        messageDiv.textContent = message;
        
        // Add to page
        document.body.appendChild(messageDiv);
        
        // Remove after 5 seconds
        setTimeout(() => {
            messageDiv.remove();
        }, 5000);
    }

    // Format file size
    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    // Export plot
    exportPlot(format = 'svg') {
        if (format === 'svg') {
            const svg = this.plot.exportAsSVG();
            if (svg) {
                this.downloadFile(svg, 'plot.svg', 'image/svg+xml');
            }
        } else if (format === 'png') {
            this.plot.exportAsPNG().then(blob => {
                if (blob) {
                    this.downloadFile(blob, 'plot.png', 'image/png');
                }
            });
        }
    }

    // Download file
    downloadFile(content, filename, mimeType) {
        const blob = new Blob([content], { type: mimeType });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    // Get current state
    getCurrentState() {
        return {
            schema: this.currentSchema,
            fileInfo: this.currentFileInfo,
            filters: this.query.getFilters(),
            derivedFields: this.query.getDerivedFields(),
            plotConfig: this.plot.plotConfig
        };
    }

    // Reset everything
    reset() {
        this.currentSchema = null;
        this.currentFileInfo = null;
        
        this.query.setData(null);
        this.plot.clear();
        
        // Hide sections
        const sections = ['filterSection', 'plotSection', 'derivedSection', 'fileInfo'];
        sections.forEach(sectionId => {
            const section = document.getElementById(sectionId);
            if (section) {
                section.style.display = 'none';
            }
        });
        
        // Clear form controls
        const selects = ['xAxis', 'yAxis', 'facetBy', 'derivedSource'];
        selects.forEach(selectId => {
            const select = document.getElementById(selectId);
            if (select) {
                select.innerHTML = '<option value="">Select field...</option>';
            }
        });
        
        // Clear derived fields
        document.getElementById('derivedFieldsList').innerHTML = '';
        document.getElementById('derivedName').value = '';
        
        // Update status
        this.updateStatus();
    }
}

// Initialize UI when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.arrowUI = new ArrowUI();
}); 