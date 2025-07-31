// Plotting Module using Observable Plot
class ArrowPlot {
    constructor() {
        this.currentPlot = null;
        this.plotContainer = null;
        this.plotData = null;
        this.plotConfig = {
            chartType: 'histogram',
            xField: null,
            yField: null,
            facetField: null,
            bins: 20,
            color: null
        };
    }

    // Initialize plot container
    init(containerId) {
        this.plotContainer = document.getElementById(containerId);
        if (!this.plotContainer) {
            console.error('Plot container not found:', containerId);
            return;
        }
    }

    // Set plot configuration
    setConfig(config) {
        this.plotConfig = { ...this.plotConfig, ...config };
    }

    // Update plot with new data
    updatePlot(data) {
        if (!data || data.length === 0) {
            this.showEmptyState();
            return;
        }

        this.plotData = data;
        this.renderPlot();
    }

    // Render the plot based on current configuration
    renderPlot() {
        if (!this.plotContainer || !this.plotData) return;

        try {
            // Clear previous plot
            this.plotContainer.innerHTML = '';

            const plotElement = this.createPlot();
            if (plotElement) {
                this.plotContainer.appendChild(plotElement);
                this.currentPlot = plotElement;
            }
        } catch (error) {
            console.error('Error rendering plot:', error);
            this.showErrorState(error.message);
        }
    }

    // Create plot based on chart type
    createPlot() {
        switch (this.plotConfig.chartType) {
            case 'histogram':
                return this.createHistogram();
            case 'heatmap':
                return this.createHeatmap();
            case 'line':
                return this.createLineChart();
            case 'scatter':
                return this.createScatterPlot();
            default:
                return this.createHistogram();
        }
    }

    // Create histogram
    createHistogram() {
        if (!this.plotConfig.xField) return null;

        const plot = Plot.plot({
            width: this.plotContainer.clientWidth - 40,
            height: this.plotContainer.clientHeight - 40,
            margin: 40,
            x: {
                grid: true,
                label: this.plotConfig.xField
            },
            y: {
                grid: true,
                label: 'Count'
            },
            marks: [
                Plot.rectY(this.plotData, Plot.binX({ y: 'count' }, { x: this.plotConfig.xField, thresholds: this.plotConfig.bins })),
                Plot.ruleY([0])
            ],
            title: `Histogram of ${this.plotConfig.xField}`
        });

        return plot;
    }

    // Create heatmap
    createHeatmap() {
        if (!this.plotConfig.xField || !this.plotConfig.yField) return null;

        const plot = Plot.plot({
            width: this.plotContainer.clientWidth - 40,
            height: this.plotContainer.clientHeight - 40,
            margin: 40,
            x: {
                grid: true,
                label: this.plotConfig.xField
            },
            y: {
                grid: true,
                label: this.plotConfig.yField
            },
            color: {
                legend: true,
                label: 'Count'
            },
            marks: [
                Plot.rect(this.plotData, Plot.bin({ fill: 'count' }, { 
                    x: this.plotConfig.xField, 
                    y: this.plotConfig.yField,
                    thresholds: this.plotConfig.bins 
                }))
            ],
            title: `Heatmap: ${this.plotConfig.xField} vs ${this.plotConfig.yField}`
        });

        return plot;
    }

    // Create line chart
    createLineChart() {
        if (!this.plotConfig.xField || !this.plotConfig.yField) return null;

        const plot = Plot.plot({
            width: this.plotContainer.clientWidth - 40,
            height: this.plotContainer.clientHeight - 40,
            margin: 40,
            x: {
                grid: true,
                label: this.plotConfig.xField
            },
            y: {
                grid: true,
                label: this.plotConfig.yField
            },
            marks: [
                Plot.line(this.plotData, { 
                    x: this.plotConfig.xField, 
                    y: this.plotConfig.yField,
                    stroke: this.plotConfig.color || 'steelblue'
                }),
                Plot.dot(this.plotData, { 
                    x: this.plotConfig.xField, 
                    y: this.plotConfig.yField,
                    fill: this.plotConfig.color || 'steelblue'
                })
            ],
            title: `Line Chart: ${this.plotConfig.yField} vs ${this.plotConfig.xField}`
        });

        return plot;
    }

    // Create scatter plot
    createScatterPlot() {
        if (!this.plotConfig.xField || !this.plotConfig.yField) return null;

        const plot = Plot.plot({
            width: this.plotContainer.clientWidth - 40,
            height: this.plotContainer.clientHeight - 40,
            margin: 40,
            x: {
                grid: true,
                label: this.plotConfig.xField
            },
            y: {
                grid: true,
                label: this.plotConfig.yField
            },
            marks: [
                Plot.dot(this.plotData, { 
                    x: this.plotConfig.xField, 
                    y: this.plotConfig.yField,
                    fill: this.plotConfig.color || 'steelblue',
                    opacity: 0.6
                })
            ],
            title: `Scatter Plot: ${this.plotConfig.yField} vs ${this.plotConfig.xField}`
        });

        return plot;
    }

    // Create faceted plot
    createFacetedPlot() {
        if (!this.plotConfig.facetField) {
            return this.createPlot();
        }

        const plot = Plot.plot({
            width: this.plotContainer.clientWidth - 40,
            height: this.plotContainer.clientHeight - 40,
            margin: 40,
            facet: {
                data: this.plotData,
                x: this.plotConfig.facetField
            },
            marks: this.getFacetedMarks(),
            title: `Faceted ${this.plotConfig.chartType}: ${this.plotConfig.facetField}`
        });

        return plot;
    }

    // Get marks for faceted plots
    getFacetedMarks() {
        switch (this.plotConfig.chartType) {
            case 'histogram':
                return [
                    Plot.rectY(this.plotData, Plot.binX({ y: 'count' }, { 
                        x: this.plotConfig.xField, 
                        thresholds: this.plotConfig.bins 
                    })),
                    Plot.ruleY([0])
                ];
            case 'scatter':
                return [
                    Plot.dot(this.plotData, { 
                        x: this.plotConfig.xField, 
                        y: this.plotConfig.yField,
                        fill: this.plotConfig.color || 'steelblue',
                        opacity: 0.6
                    })
                ];
            case 'line':
                return [
                    Plot.line(this.plotData, { 
                        x: this.plotConfig.xField, 
                        y: this.plotConfig.yField,
                        stroke: this.plotConfig.color || 'steelblue'
                    }),
                    Plot.dot(this.plotData, { 
                        x: this.plotConfig.xField, 
                        y: this.plotConfig.yField,
                        fill: this.plotConfig.color || 'steelblue'
                    })
                ];
            default:
                return [];
        }
    }

    // Show empty state
    showEmptyState() {
        if (!this.plotContainer) return;

        this.plotContainer.innerHTML = `
            <div class="plot-placeholder">
                <svg class="placeholder-icon" viewBox="0 0 24 24" width="120" height="120">
                    <path d="M3 3v18h18V3H3zm16 16H5V5h14v14z"/>
                    <path d="M7 7h10v2H7V7zm0 4h10v2H7v-2zm0 4h7v2H7v-2z"/>
                </svg>
                <h3>No data to plot</h3>
                <p>Select fields and configure your plot to visualize the data</p>
            </div>
        `;
    }

    // Show error state
    showErrorState(message) {
        if (!this.plotContainer) return;

        this.plotContainer.innerHTML = `
            <div class="error">
                <h3>Plot Error</h3>
                <p>${message}</p>
                <p>Please check your data and plot configuration.</p>
            </div>
        `;
    }

    // Export plot as SVG
    exportAsSVG() {
        if (!this.currentPlot) return null;

        try {
            const svg = this.currentPlot.querySelector('svg');
            if (svg) {
                const serializer = new XMLSerializer();
                return serializer.serializeToString(svg);
            }
        } catch (error) {
            console.error('Error exporting plot:', error);
        }
        return null;
    }

    // Export plot as PNG
    async exportAsPNG() {
        if (!this.currentPlot) return null;

        try {
            const svg = this.currentPlot.querySelector('svg');
            if (svg) {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                const img = new Image();
                
                const svgData = new XMLSerializer().serializeToString(svg);
                const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
                const url = URL.createObjectURL(svgBlob);
                
                return new Promise((resolve, reject) => {
                    img.onload = () => {
                        canvas.width = img.width;
                        canvas.height = img.height;
                        ctx.drawImage(img, 0, 0);
                        canvas.toBlob(resolve, 'image/png');
                        URL.revokeObjectURL(url);
                    };
                    img.onerror = reject;
                    img.src = url;
                });
            }
        } catch (error) {
            console.error('Error exporting plot as PNG:', error);
        }
        return null;
    }

    // Get plot statistics
    getPlotStatistics() {
        if (!this.plotData) return null;

        const stats = {
            dataPoints: this.plotData.length,
            chartType: this.plotConfig.chartType,
            fields: {
                x: this.plotConfig.xField,
                y: this.plotConfig.yField,
                facet: this.plotConfig.facetField
            }
        };

        if (this.plotConfig.xField) {
            const xValues = this.plotData.map(d => d[this.plotConfig.xField]).filter(v => v != null);
            if (xValues.length > 0) {
                stats.xStats = {
                    min: Math.min(...xValues),
                    max: Math.max(...xValues),
                    mean: xValues.reduce((a, b) => a + b, 0) / xValues.length
                };
            }
        }

        if (this.plotConfig.yField) {
            const yValues = this.plotData.map(d => d[this.plotConfig.yField]).filter(v => v != null);
            if (yValues.length > 0) {
                stats.yStats = {
                    min: Math.min(...yValues),
                    max: Math.max(...yValues),
                    mean: yValues.reduce((a, b) => a + b, 0) / yValues.length
                };
            }
        }

        return stats;
    }

    // Resize plot
    resize() {
        if (this.currentPlot) {
            this.renderPlot();
        }
    }

    // Clear plot
    clear() {
        if (this.plotContainer) {
            this.plotContainer.innerHTML = '';
        }
        this.currentPlot = null;
        this.plotData = null;
    }
}

// Export for use in other modules
window.ArrowPlot = ArrowPlot; 