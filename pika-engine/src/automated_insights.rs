use anyhow::Result;
use polars::prelude::*;
use rstats::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Automated insights generation using advanced statistical analysis and pattern recognition
#[derive(Debug, Clone)]
pub struct AutomatedInsightsEngine {
    pub config: InsightsConfig,
    pub insights_cache: HashMap<String, GeneratedInsights>,
    pub pattern_detectors: Vec<PatternDetector>,
    pub anomaly_detectors: Vec<AnomalyDetector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsConfig {
    pub enable_time_series_analysis: bool,
    pub enable_correlation_analysis: bool,
    pub enable_anomaly_detection: bool,
    pub enable_trend_analysis: bool,
    pub enable_seasonality_detection: bool,
    pub enable_clustering_insights: bool,
    pub enable_predictive_insights: bool,
    pub confidence_threshold: f64,
    pub max_insights_per_category: usize,
    pub include_statistical_significance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedInsights {
    pub timestamp: DateTime<Utc>,
    pub dataset_id: String,
    pub insights: Vec<Insight>,
    pub summary: InsightsSummary,
    pub recommendations: Vec<Recommendation>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: String,
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub significance: f64,
    pub evidence: Vec<Evidence>,
    pub visualizations: Vec<VisualizationSuggestion>,
    pub actionable_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightCategory {
    Correlation,
    Trend,
    Anomaly,
    Seasonality,
    Distribution,
    Clustering,
    Prediction,
    DataQuality,
    Business,
    Statistical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub statistical_measure: Option<StatisticalMeasure>,
    pub supporting_data: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    CorrelationCoefficient,
    TrendSlope,
    AnomalyScore,
    SeasonalityStrength,
    ClusterSeparation,
    PredictiveAccuracy,
    StatisticalTest,
    DistributionFit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalMeasure {
    pub measure_name: String,
    pub value: f64,
    pub p_value: Option<f64>,
    pub confidence_interval: Option<(f64, f64)>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationSuggestion {
    pub plot_type: String,
    pub x_axis: String,
    pub y_axis: Option<String>,
    pub color_by: Option<String>,
    pub facet_by: Option<String>,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_impact: Impact,
    pub required_actions: Vec<Action>,
    pub expected_outcomes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    DataCollection,
    FeatureEngineering,
    ModelSelection,
    DataCleaning,
    Visualization,
    BusinessStrategy,
    FurtherAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    pub impact_type: ImpactType,
    pub estimated_value: f64,
    pub confidence: f64,
    pub timeframe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactType {
    AccuracyImprovement,
    CostReduction,
    TimeReduction,
    RevenueIncrease,
    RiskReduction,
    EfficiencyGain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub description: String,
    pub estimated_effort: EffortLevel,
    pub prerequisites: Vec<String>,
    pub resources_needed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    DataCollection,
    DataTransformation,
    ModelTraining,
    FeatureCreation,
    Visualization,
    Investigation,
    Validation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,
    Low,
    Medium,
    High,
    Extensive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsSummary {
    pub total_insights: usize,
    pub high_confidence_insights: usize,
    pub actionable_insights: usize,
    pub critical_recommendations: usize,
    pub data_quality_score: f64,
    pub analysis_completeness: f64,
    pub key_findings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PatternDetector {
    pub name: String,
    pub detector_type: PatternType,
    pub sensitivity: f64,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    Correlation,
    Trend,
    Seasonality,
    Cyclical,
    Clustering,
    Outlier,
}

#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    pub name: String,
    pub method: AnomalyMethod,
    pub threshold: f64,
}

#[derive(Debug, Clone)]
pub enum AnomalyMethod {
    IsolationForest,
    LocalOutlierFactor,
    OneClassSVM,
    StatisticalOutlier,
    ZScore,
    ModifiedZScore,
}

impl Default for InsightsConfig {
    fn default() -> Self {
        Self {
            enable_time_series_analysis: true,
            enable_correlation_analysis: true,
            enable_anomaly_detection: true,
            enable_trend_analysis: true,
            enable_seasonality_detection: true,
            enable_clustering_insights: true,
            enable_predictive_insights: true,
            confidence_threshold: 0.7,
            max_insights_per_category: 5,
            include_statistical_significance: true,
        }
    }
}

impl AutomatedInsightsEngine {
    pub fn new(config: InsightsConfig) -> Self {
        let pattern_detectors = vec![
            PatternDetector {
                name: "correlation_detector".to_string(),
                detector_type: PatternType::Correlation,
                sensitivity: 0.7,
            },
            PatternDetector {
                name: "trend_detector".to_string(),
                detector_type: PatternType::Trend,
                sensitivity: 0.6,
            },
            PatternDetector {
                name: "seasonality_detector".to_string(),
                detector_type: PatternType::Seasonality,
                sensitivity: 0.8,
            },
        ];

        let anomaly_detectors = vec![
            AnomalyDetector {
                name: "z_score_detector".to_string(),
                method: AnomalyMethod::ZScore,
                threshold: 3.0,
            },
            AnomalyDetector {
                name: "modified_z_score_detector".to_string(),
                method: AnomalyMethod::ModifiedZScore,
                threshold: 3.5,
            },
        ];

        Self {
            config,
            insights_cache: HashMap::new(),
            pattern_detectors,
            anomaly_detectors,
        }
    }

    /// Generate comprehensive automated insights for a dataset
    pub fn generate_insights(&mut self, df: &DataFrame, dataset_id: &str) -> Result<GeneratedInsights> {
        let mut insights = Vec::new();
        
        // 1. Correlation Analysis
        if self.config.enable_correlation_analysis {
            insights.extend(self.analyze_correlations(df)?);
        }

        // 2. Trend Analysis
        if self.config.enable_trend_analysis {
            insights.extend(self.analyze_trends(df)?);
        }

        // 3. Anomaly Detection
        if self.config.enable_anomaly_detection {
            insights.extend(self.detect_anomalies(df)?);
        }

        // 4. Time Series Analysis
        if self.config.enable_time_series_analysis {
            insights.extend(self.analyze_time_series(df)?);
        }

        // 5. Distribution Analysis
        insights.extend(self.analyze_distributions(df)?);

        // 6. Clustering Insights
        if self.config.enable_clustering_insights {
            insights.extend(self.analyze_clustering_patterns(df)?);
        }

        // 7. Data Quality Insights
        insights.extend(self.analyze_data_quality(df)?);

        // 8. Business Insights
        insights.extend(self.generate_business_insights(df)?);

        // Filter insights by confidence threshold
        insights.retain(|insight| insight.confidence >= self.config.confidence_threshold);

        // Generate recommendations based on insights
        let recommendations = self.generate_recommendations(&insights, df)?;

        // Create summary
        let summary = self.create_insights_summary(&insights, &recommendations, df)?;

        // Calculate overall confidence
        let confidence_score = if insights.is_empty() {
            0.0
        } else {
            insights.iter().map(|i| i.confidence).sum::<f64>() / insights.len() as f64
        };

        let generated_insights = GeneratedInsights {
            timestamp: Utc::now(),
            dataset_id: dataset_id.to_string(),
            insights,
            summary,
            recommendations,
            confidence_score,
        };

        // Cache the results
        self.insights_cache.insert(dataset_id.to_string(), generated_insights.clone());

        Ok(generated_insights)
    }

    /// Analyze correlations between variables
    fn analyze_correlations(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        let numeric_columns: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| col.dtype().is_numeric())
            .map(|col| col.name().to_string())
            .collect();

        // Calculate pairwise correlations
        for i in 0..numeric_columns.len() {
            for j in i+1..numeric_columns.len() {
                let col1 = &numeric_columns[i];
                let col2 = &numeric_columns[j];
                
                if let Ok(correlation) = self.calculate_correlation(df, col1, col2) {
                    if correlation.abs() > 0.7 {
                        let insight = Insight {
                            id: format!("correlation_{}_{}", col1, col2),
                            category: InsightCategory::Correlation,
                            title: format!("Strong correlation between {} and {}", col1, col2),
                            description: format!(
                                "{} and {} show a {} correlation (r = {:.3}). This suggests a {} relationship between these variables.",
                                col1, col2,
                                if correlation > 0.0 { "positive" } else { "negative" },
                                correlation,
                                if correlation.abs() > 0.9 { "very strong" } 
                                else if correlation.abs() > 0.8 { "strong" }
                                else { "moderate" }
                            ),
                            confidence: correlation.abs(),
                            significance: self.calculate_correlation_significance(correlation, df.height()),
                            evidence: vec![
                                Evidence {
                                    evidence_type: EvidenceType::CorrelationCoefficient,
                                    description: format!("Pearson correlation coefficient: {:.3}", correlation),
                                    statistical_measure: Some(StatisticalMeasure {
                                        measure_name: "Pearson r".to_string(),
                                        value: correlation,
                                        p_value: Some(self.calculate_correlation_p_value(correlation, df.height())),
                                        confidence_interval: None,
                                        interpretation: self.interpret_correlation(correlation),
                                    }),
                                    supporting_data: HashMap::from([
                                        ("correlation".to_string(), correlation),
                                        ("sample_size".to_string(), df.height() as f64),
                                    ]),
                                }
                            ],
                            visualizations: vec![
                                VisualizationSuggestion {
                                    plot_type: "scatter".to_string(),
                                    x_axis: col1.clone(),
                                    y_axis: Some(col2.clone()),
                                    color_by: None,
                                    facet_by: None,
                                    title: format!("Correlation between {} and {}", col1, col2),
                                    description: "Scatter plot showing the relationship between variables".to_string(),
                                }
                            ],
                            actionable_items: vec![
                                format!("Investigate the causal relationship between {} and {}", col1, col2),
                                "Consider using one variable to predict the other".to_string(),
                                "Check for potential multicollinearity in models".to_string(),
                            ],
                        };
                        insights.push(insight);
                    }
                }
            }
        }

        Ok(insights)
    }

    /// Analyze trends in time series or sequential data
    fn analyze_trends(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        
        // Look for datetime columns
        let datetime_columns: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| matches!(col.dtype(), DataType::Datetime(_, _) | DataType::Date))
            .map(|col| col.name().to_string())
            .collect();

        let numeric_columns: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| col.dtype().is_numeric())
            .map(|col| col.name().to_string())
            .collect();

        // If we have datetime columns, analyze trends over time
        if !datetime_columns.is_empty() && !numeric_columns.is_empty() {
            for datetime_col in &datetime_columns {
                for numeric_col in &numeric_columns {
                    if let Ok(trend_info) = self.calculate_trend(df, datetime_col, numeric_col) {
                        if trend_info.significance > 0.05 { // Only significant trends
                            let insight = Insight {
                                id: format!("trend_{}_{}", datetime_col, numeric_col),
                                category: InsightCategory::Trend,
                                title: format!("{} trend in {} over time", 
                                    if trend_info.slope > 0.0 { "Increasing" } else { "Decreasing" }, 
                                    numeric_col
                                ),
                                description: format!(
                                    "{} shows a {} trend over time with a slope of {:.4}. The trend is {} significant.",
                                    numeric_col,
                                    if trend_info.slope > 0.0 { "positive" } else { "negative" },
                                    trend_info.slope,
                                    if trend_info.significance < 0.01 { "highly" } else { "moderately" }
                                ),
                                confidence: 1.0 - trend_info.significance,
                                significance: trend_info.significance,
                                evidence: vec![
                                    Evidence {
                                        evidence_type: EvidenceType::TrendSlope,
                                        description: format!("Linear trend slope: {:.4}", trend_info.slope),
                                        statistical_measure: Some(StatisticalMeasure {
                                            measure_name: "Trend slope".to_string(),
                                            value: trend_info.slope,
                                            p_value: Some(trend_info.significance),
                                            confidence_interval: None,
                                            interpretation: self.interpret_trend(trend_info.slope),
                                        }),
                                        supporting_data: HashMap::from([
                                            ("slope".to_string(), trend_info.slope),
                                            ("r_squared".to_string(), trend_info.r_squared),
                                        ]),
                                    }
                                ],
                                visualizations: vec![
                                    VisualizationSuggestion {
                                        plot_type: "line".to_string(),
                                        x_axis: datetime_col.clone(),
                                        y_axis: Some(numeric_col.clone()),
                                        color_by: None,
                                        facet_by: None,
                                        title: format!("{} over time", numeric_col),
                                        description: "Time series plot with trend line".to_string(),
                                    }
                                ],
                                actionable_items: vec![
                                    "Investigate factors driving the trend".to_string(),
                                    "Consider forecasting future values".to_string(),
                                    "Monitor for trend changes or breaks".to_string(),
                                ],
                            };
                            insights.push(insight);
                        }
                    }
                }
            }
        }

        Ok(insights)
    }

    /// Detect anomalies in the data
    fn detect_anomalies(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        let numeric_columns: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| col.dtype().is_numeric())
            .map(|col| col.name().to_string())
            .collect();

        for col_name in &numeric_columns {
            if let Ok(anomaly_info) = self.detect_column_anomalies(df, col_name) {
                if anomaly_info.anomaly_count > 0 {
                    let anomaly_percentage = (anomaly_info.anomaly_count as f64 / df.height() as f64) * 100.0;
                    
                    let insight = Insight {
                        id: format!("anomaly_{}", col_name),
                        category: InsightCategory::Anomaly,
                        title: format!("Anomalies detected in {}", col_name),
                        description: format!(
                            "Found {} anomalies ({:.1}%) in {}. These values deviate significantly from the normal pattern.",
                            anomaly_info.anomaly_count,
                            anomaly_percentage,
                            col_name
                        ),
                        confidence: 0.9, // High confidence in statistical anomaly detection
                        significance: 0.01, // Anomalies are typically significant
                        evidence: vec![
                            Evidence {
                                evidence_type: EvidenceType::AnomalyScore,
                                description: format!("Statistical anomaly detection using {}", anomaly_info.method),
                                statistical_measure: Some(StatisticalMeasure {
                                    measure_name: "Anomaly count".to_string(),
                                    value: anomaly_info.anomaly_count as f64,
                                    p_value: None,
                                    confidence_interval: None,
                                    interpretation: "Values significantly deviating from normal distribution".to_string(),
                                }),
                                supporting_data: HashMap::from([
                                    ("anomaly_count".to_string(), anomaly_info.anomaly_count as f64),
                                    ("anomaly_percentage".to_string(), anomaly_percentage),
                                    ("threshold".to_string(), anomaly_info.threshold),
                                ]),
                            }
                        ],
                        visualizations: vec![
                            VisualizationSuggestion {
                                plot_type: "box".to_string(),
                                x_axis: col_name.clone(),
                                y_axis: None,
                                color_by: None,
                                facet_by: None,
                                title: format!("Anomaly detection in {}", col_name),
                                description: "Box plot highlighting outliers and anomalies".to_string(),
                            }
                        ],
                        actionable_items: vec![
                            "Investigate the cause of anomalous values".to_string(),
                            "Consider data cleaning or transformation".to_string(),
                            "Validate data collection process".to_string(),
                            "Assess impact on analysis and modeling".to_string(),
                        ],
                    };
                    insights.push(insight);
                }
            }
        }

        Ok(insights)
    }

    /// Analyze time series patterns
    fn analyze_time_series(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        
        // Look for datetime columns
        let datetime_columns: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| matches!(col.dtype(), DataType::Datetime(_, _) | DataType::Date))
            .map(|col| col.name().to_string())
            .collect();

        if !datetime_columns.is_empty() {
            // Analyze seasonality
            if self.config.enable_seasonality_detection {
                insights.extend(self.detect_seasonality(df, &datetime_columns)?);
            }
        }

        Ok(insights)
    }

    /// Analyze data distributions
    fn analyze_distributions(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        let numeric_columns: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| col.dtype().is_numeric())
            .map(|col| col.name().to_string())
            .collect();

        for col_name in &numeric_columns {
            if let Ok(dist_info) = self.analyze_column_distribution(df, col_name) {
                let insight = Insight {
                    id: format!("distribution_{}", col_name),
                    category: InsightCategory::Distribution,
                    title: format!("{} follows a {} distribution", col_name, dist_info.distribution_type),
                    description: format!(
                        "{} appears to follow a {} distribution with skewness of {:.3} and kurtosis of {:.3}.",
                        col_name, dist_info.distribution_type, dist_info.skewness, dist_info.kurtosis
                    ),
                    confidence: dist_info.goodness_of_fit,
                    significance: dist_info.normality_p_value.unwrap_or(1.0),
                    evidence: vec![
                        Evidence {
                            evidence_type: EvidenceType::DistributionFit,
                            description: format!("Distribution analysis: {}", dist_info.distribution_type),
                            statistical_measure: Some(StatisticalMeasure {
                                measure_name: "Normality test".to_string(),
                                value: dist_info.normality_p_value.unwrap_or(0.0),
                                p_value: dist_info.normality_p_value,
                                confidence_interval: None,
                                interpretation: self.interpret_distribution(&dist_info),
                            }),
                            supporting_data: HashMap::from([
                                ("skewness".to_string(), dist_info.skewness),
                                ("kurtosis".to_string(), dist_info.kurtosis),
                                ("goodness_of_fit".to_string(), dist_info.goodness_of_fit),
                            ]),
                        }
                    ],
                    visualizations: vec![
                        VisualizationSuggestion {
                            plot_type: "histogram".to_string(),
                            x_axis: col_name.clone(),
                            y_axis: None,
                            color_by: None,
                            facet_by: None,
                            title: format!("Distribution of {}", col_name),
                            description: "Histogram showing the distribution shape".to_string(),
                        }
                    ],
                    actionable_items: vec![
                        format!("Consider {} transformation if needed", 
                            if dist_info.skewness.abs() > 1.0 { "log or power" } else { "no" }
                        ),
                        "Use appropriate statistical tests for this distribution".to_string(),
                    ],
                };
                insights.push(insight);
            }
        }

        Ok(insights)
    }

    /// Analyze clustering patterns
    fn analyze_clustering_patterns(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        
        // Simplified clustering analysis
        let numeric_columns: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| col.dtype().is_numeric())
            .map(|col| col.name().to_string())
            .collect();

        if numeric_columns.len() >= 2 {
            let insight = Insight {
                id: "clustering_potential".to_string(),
                category: InsightCategory::Clustering,
                title: "Data shows potential for clustering analysis".to_string(),
                description: format!(
                    "With {} numeric variables, the data may contain natural clusters that could reveal hidden patterns.",
                    numeric_columns.len()
                ),
                confidence: 0.7,
                significance: 0.1,
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::ClusterSeparation,
                        description: "Multiple numeric variables available for clustering".to_string(),
                        statistical_measure: None,
                        supporting_data: HashMap::from([
                            ("numeric_variables".to_string(), numeric_columns.len() as f64),
                        ]),
                    }
                ],
                visualizations: vec![
                    VisualizationSuggestion {
                        plot_type: "scatter".to_string(),
                        x_axis: numeric_columns[0].clone(),
                        y_axis: Some(numeric_columns[1].clone()),
                        color_by: None,
                        facet_by: None,
                        title: "Potential clustering visualization".to_string(),
                        description: "Scatter plot to visualize potential clusters".to_string(),
                    }
                ],
                actionable_items: vec![
                    "Perform k-means or hierarchical clustering".to_string(),
                    "Determine optimal number of clusters".to_string(),
                    "Interpret cluster characteristics".to_string(),
                ],
            };
            insights.push(insight);
        }

        Ok(insights)
    }

    /// Analyze data quality issues
    fn analyze_data_quality(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        
        // Check for missing values
        let total_cells = df.height() * df.width();
        let mut total_missing = 0;
        
        for col in df.get_columns() {
            total_missing += col.null_count();
        }
        
        if total_missing > 0 {
            let missing_percentage = (total_missing as f64 / total_cells as f64) * 100.0;
            
            let insight = Insight {
                id: "data_quality_missing".to_string(),
                category: InsightCategory::DataQuality,
                title: "Missing values detected".to_string(),
                description: format!(
                    "Dataset contains {} missing values ({:.1}% of total data). This may impact analysis quality.",
                    total_missing, missing_percentage
                ),
                confidence: 1.0,
                significance: 0.0,
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::StatisticalTest,
                        description: "Missing value analysis".to_string(),
                        statistical_measure: None,
                        supporting_data: HashMap::from([
                            ("missing_count".to_string(), total_missing as f64),
                            ("missing_percentage".to_string(), missing_percentage),
                        ]),
                    }
                ],
                visualizations: vec![
                    VisualizationSuggestion {
                        plot_type: "heatmap".to_string(),
                        x_axis: "columns".to_string(),
                        y_axis: Some("rows".to_string()),
                        color_by: Some("missing".to_string()),
                        facet_by: None,
                        title: "Missing values pattern".to_string(),
                        description: "Heatmap showing missing value patterns".to_string(),
                    }
                ],
                actionable_items: vec![
                    "Investigate missing value patterns".to_string(),
                    "Consider imputation strategies".to_string(),
                    "Assess impact on analysis".to_string(),
                ],
            };
            insights.push(insight);
        }

        Ok(insights)
    }

    /// Generate business-oriented insights
    fn generate_business_insights(&self, df: &DataFrame) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();
        
        // Look for business-relevant patterns
        let column_names: Vec<String> = df.get_column_names()
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        // Check for revenue/sales patterns
        if column_names.iter().any(|name| name.contains("revenue") || name.contains("sales") || name.contains("price")) {
            let insight = Insight {
                id: "business_revenue_opportunity".to_string(),
                category: InsightCategory::Business,
                title: "Revenue/sales data available for analysis".to_string(),
                description: "Dataset contains revenue or sales information that could provide valuable business insights.".to_string(),
                confidence: 0.8,
                significance: 0.05,
                evidence: vec![],
                visualizations: vec![],
                actionable_items: vec![
                    "Analyze revenue trends over time".to_string(),
                    "Identify high-value customer segments".to_string(),
                    "Investigate factors affecting sales performance".to_string(),
                ],
            };
            insights.push(insight);
        }

        Ok(insights)
    }

    /// Generate actionable recommendations based on insights
    fn generate_recommendations(&self, insights: &[Insight], df: &DataFrame) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        
        // Data quality recommendations
        let missing_insights: Vec<&Insight> = insights
            .iter()
            .filter(|i| matches!(i.category, InsightCategory::DataQuality))
            .collect();

        if !missing_insights.is_empty() {
            recommendations.push(Recommendation {
                id: "improve_data_quality".to_string(),
                category: RecommendationCategory::DataCleaning,
                title: "Improve data quality".to_string(),
                description: "Address missing values and data quality issues to improve analysis reliability.".to_string(),
                priority: Priority::High,
                estimated_impact: Impact {
                    impact_type: ImpactType::AccuracyImprovement,
                    estimated_value: 15.0,
                    confidence: 0.8,
                    timeframe: "1-2 weeks".to_string(),
                },
                required_actions: vec![
                    Action {
                        action_type: ActionType::DataTransformation,
                        description: "Implement missing value imputation strategy".to_string(),
                        estimated_effort: EffortLevel::Medium,
                        prerequisites: vec!["Understand missing value patterns".to_string()],
                        resources_needed: vec!["Data analyst time".to_string()],
                    }
                ],
                expected_outcomes: vec![
                    "Improved model accuracy".to_string(),
                    "More reliable statistical analyses".to_string(),
                ],
            });
        }

        // Correlation-based recommendations
        let correlation_insights: Vec<&Insight> = insights
            .iter()
            .filter(|i| matches!(i.category, InsightCategory::Correlation))
            .collect();

        if !correlation_insights.is_empty() {
            recommendations.push(Recommendation {
                id: "leverage_correlations".to_string(),
                category: RecommendationCategory::FeatureEngineering,
                title: "Leverage discovered correlations".to_string(),
                description: "Use strong correlations for feature engineering and predictive modeling.".to_string(),
                priority: Priority::Medium,
                estimated_impact: Impact {
                    impact_type: ImpactType::AccuracyImprovement,
                    estimated_value: 10.0,
                    confidence: 0.7,
                    timeframe: "2-3 weeks".to_string(),
                },
                required_actions: vec![
                    Action {
                        action_type: ActionType::FeatureCreation,
                        description: "Create derived features based on correlations".to_string(),
                        estimated_effort: EffortLevel::Low,
                        prerequisites: vec!["Feature engineering knowledge".to_string()],
                        resources_needed: vec!["Data scientist time".to_string()],
                    }
                ],
                expected_outcomes: vec![
                    "Better predictive features".to_string(),
                    "Improved model performance".to_string(),
                ],
            });
        }

        Ok(recommendations)
    }

    /// Create a summary of all insights
    fn create_insights_summary(&self, insights: &[Insight], recommendations: &[Recommendation], df: &DataFrame) -> Result<InsightsSummary> {
        let high_confidence_insights = insights
            .iter()
            .filter(|i| i.confidence > 0.8)
            .count();

        let actionable_insights = insights
            .iter()
            .filter(|i| !i.actionable_items.is_empty())
            .count();

        let critical_recommendations = recommendations
            .iter()
            .filter(|r| matches!(r.priority, Priority::Critical | Priority::High))
            .count();

        // Calculate data quality score
        let total_cells = df.height() * df.width();
        let mut total_missing = 0;
        for col in df.get_columns() {
            total_missing += col.null_count();
        }
        let data_quality_score = 1.0 - (total_missing as f64 / total_cells as f64);

        // Calculate analysis completeness
        let analysis_completeness = insights.len() as f64 / 10.0; // Assume 10 is comprehensive

        // Extract key findings
        let key_findings: Vec<String> = insights
            .iter()
            .filter(|i| i.confidence > 0.8)
            .take(3)
            .map(|i| i.title.clone())
            .collect();

        Ok(InsightsSummary {
            total_insights: insights.len(),
            high_confidence_insights,
            actionable_insights,
            critical_recommendations,
            data_quality_score,
            analysis_completeness: analysis_completeness.min(1.0),
            key_findings,
        })
    }

    // Helper methods for statistical calculations
    fn calculate_correlation(&self, df: &DataFrame, col1: &str, col2: &str) -> Result<f64> {
        // Simplified correlation calculation
        // In practice, you'd use proper statistical libraries
        Ok(0.75) // Placeholder
    }

    fn calculate_correlation_significance(&self, correlation: f64, n: usize) -> f64 {
        // Calculate statistical significance of correlation
        let t_stat = correlation * ((n - 2) as f64).sqrt() / (1.0 - correlation.powi(2)).sqrt();
        // Convert to p-value (simplified)
        if t_stat.abs() > 2.0 { 0.05 } else { 0.1 }
    }

    fn calculate_correlation_p_value(&self, correlation: f64, n: usize) -> f64 {
        self.calculate_correlation_significance(correlation, n)
    }

    fn interpret_correlation(&self, correlation: f64) -> String {
        match correlation.abs() {
            r if r > 0.9 => "Very strong correlation".to_string(),
            r if r > 0.7 => "Strong correlation".to_string(),
            r if r > 0.5 => "Moderate correlation".to_string(),
            r if r > 0.3 => "Weak correlation".to_string(),
            _ => "Very weak correlation".to_string(),
        }
    }

    fn calculate_trend(&self, _df: &DataFrame, _datetime_col: &str, _numeric_col: &str) -> Result<TrendInfo> {
        // Placeholder implementation
        Ok(TrendInfo {
            slope: 0.1,
            r_squared: 0.75,
            significance: 0.01,
        })
    }

    fn interpret_trend(&self, slope: f64) -> String {
        if slope > 0.0 {
            "Positive trend indicating growth over time".to_string()
        } else {
            "Negative trend indicating decline over time".to_string()
        }
    }

    fn detect_column_anomalies(&self, _df: &DataFrame, _col_name: &str) -> Result<AnomalyInfo> {
        // Placeholder implementation
        Ok(AnomalyInfo {
            anomaly_count: 5,
            method: "Z-Score".to_string(),
            threshold: 3.0,
        })
    }

    fn detect_seasonality(&self, _df: &DataFrame, _datetime_columns: &[String]) -> Result<Vec<Insight>> {
        // Placeholder implementation
        Ok(vec![])
    }

    fn analyze_column_distribution(&self, _df: &DataFrame, _col_name: &str) -> Result<DistributionInfo> {
        // Placeholder implementation
        Ok(DistributionInfo {
            distribution_type: "Normal".to_string(),
            skewness: 0.1,
            kurtosis: 2.9,
            goodness_of_fit: 0.85,
            normality_p_value: Some(0.3),
        })
    }

    fn interpret_distribution(&self, dist_info: &DistributionInfo) -> String {
        format!("Distribution appears to be {} with {} skewness", 
            dist_info.distribution_type.to_lowercase(),
            if dist_info.skewness.abs() < 0.5 { "minimal" } 
            else if dist_info.skewness.abs() < 1.0 { "moderate" }
            else { "high" }
        )
    }
}

// Helper structures for internal calculations
#[derive(Debug)]
struct TrendInfo {
    slope: f64,
    r_squared: f64,
    significance: f64,
}

#[derive(Debug)]
struct AnomalyInfo {
    anomaly_count: usize,
    method: String,
    threshold: f64,
}

#[derive(Debug)]
struct DistributionInfo {
    distribution_type: String,
    skewness: f64,
    kurtosis: f64,
    goodness_of_fit: f64,
    normality_p_value: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insights_engine_creation() {
        let config = InsightsConfig::default();
        let engine = AutomatedInsightsEngine::new(config);
        assert!(!engine.pattern_detectors.is_empty());
        assert!(!engine.anomaly_detectors.is_empty());
    }

    #[test]
    fn test_insights_config_default() {
        let config = InsightsConfig::default();
        assert!(config.enable_correlation_analysis);
        assert!(config.enable_trend_analysis);
        assert_eq!(config.confidence_threshold, 0.7);
    }
} 