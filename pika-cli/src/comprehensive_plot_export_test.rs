//! Comprehensive plot export test that generates actual plot images
//! This creates real plot images with various configurations for visual inspection

use plotters::prelude::*;
use std::path::Path;
use std::fs;

/// Configuration for a plot export test
#[derive(Clone)]
struct PlotTestConfig {
    name: String,
    plot_type: PlotType,
    with_legend: bool,
    theme: Theme,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug)]
enum PlotType {
    Line,
    Bar,
    Scatter,
    Histogram,
    Area,
    // More complex plots to add later:
    // Box,
    // Violin,
    // Heatmap,
    // Radar,
}

#[derive(Clone, Copy)]
enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn background_color(&self) -> RGBColor {
        match self {
            Theme::Light => WHITE,
            Theme::Dark => RGBColor(30, 30, 30),
        }
    }
    
    fn foreground_color(&self) -> RGBColor {
        match self {
            Theme::Light => BLACK,
            Theme::Dark => RGBColor(220, 220, 220),
        }
    }
    
    fn grid_color(&self) -> RGBColor {
        match self {
            Theme::Light => RGBColor(200, 200, 200),
            Theme::Dark => RGBColor(80, 80, 80),
        }
    }
    
    fn plot_colors(&self) -> Vec<RGBColor> {
        match self {
            Theme::Light => vec![
                RGBColor(31, 119, 180),   // Blue
                RGBColor(255, 127, 14),   // Orange
                RGBColor(44, 160, 44),    // Green
                RGBColor(214, 39, 40),    // Red
                RGBColor(148, 103, 189),  // Purple
            ],
            Theme::Dark => vec![
                RGBColor(77, 172, 238),   // Light Blue
                RGBColor(255, 178, 102),  // Light Orange
                RGBColor(119, 221, 119),  // Light Green
                RGBColor(255, 128, 128),  // Light Red
                RGBColor(209, 179, 255),  // Light Purple
            ],
        }
    }
}

/// Generate sample data for plotting
fn generate_sample_data() -> Vec<(f64, f64)> {
    (0..50)
        .map(|i| {
            let x = i as f64 / 10.0;
            let y = (x * 0.8).sin() * 10.0 + 20.0 + (i as f64 * 0.1).cos() * 5.0;
            (x, y)
        })
        .collect()
}

/// Generate multiple series for comparison
fn generate_multi_series() -> Vec<Vec<(f64, f64)>> {
    vec![
        (0..30).map(|i| {
            let x = i as f64 / 5.0;
            (x, x.sin() * 10.0 + 15.0)
        }).collect(),
        (0..30).map(|i| {
            let x = i as f64 / 5.0;
            (x, x.cos() * 8.0 + 12.0)
        }).collect(),
        (0..30).map(|i| {
            let x = i as f64 / 5.0;
            (x, (x * 0.5).sin() * 12.0 + 18.0)
        }).collect(),
    ]
}

/// Generate histogram data
fn generate_histogram_data() -> Vec<f64> {
    use rand::Rng;
    use rand::thread_rng;
    
    let mut rng = thread_rng();
    
    (0..1000)
        .map(|_| {
            // Generate a normal-like distribution using Box-Muller transform
            let u1 = rng.gen::<f64>();
            let u2 = rng.gen::<f64>();
            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            50.0 + z0 * 15.0 // mean=50, std_dev=15
        })
        .collect()
}

/// Create a line plot
fn create_line_plot(path: &Path, config: &PlotTestConfig, data: &[Vec<(f64, f64)>]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
    root.fill(&config.theme.background_color())?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&config.name, ("sans-serif", 40).into_font().color(&config.theme.foreground_color()))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..6f64, 0f64..35f64)?;
    
    chart
        .configure_mesh()
        .x_desc("X Axis")
        .y_desc("Y Axis")
        .label_style(("sans-serif", 15).into_font().color(&config.theme.foreground_color()))
        .axis_style(&config.theme.foreground_color())
        .light_line_style(&config.theme.grid_color())
        .draw()?;
    
    let colors = config.theme.plot_colors();
    let series_names = vec!["Series A", "Series B", "Series C"];
    
    for (idx, series) in data.iter().enumerate() {
        let color = colors[idx % colors.len()];
        let series_style = ShapeStyle::from(&color).stroke_width(2);
        
        let series_plot = chart
            .draw_series(LineSeries::new(
                series.iter().cloned(),
                series_style.clone(),
            ))?;
            
        if config.with_legend {
            series_plot.label(series_names[idx])
                .legend(move |(x, y)| Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], series_style.clone()));
        }
    }
    
    if config.with_legend {
        chart
            .configure_series_labels()
            .background_style(&config.theme.background_color().mix(0.8))
            .border_style(&config.theme.foreground_color())
            .label_font(("sans-serif", 15).into_font().color(&config.theme.foreground_color()))
            .draw()?;
    }
    
    root.present()?;
    Ok(())
}

/// Create a bar plot
fn create_bar_plot(path: &Path, config: &PlotTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
    root.fill(&config.theme.background_color())?;
    
    let data = vec![
        ("Q1", 45.0),
        ("Q2", 52.0),
        ("Q3", 48.0),
        ("Q4", 61.0),
    ];
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&config.name, ("sans-serif", 40).into_font().color(&config.theme.foreground_color()))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(
            0..data.len(),
            0f64..70f64,
        )?;
    
    // Configure chart
    chart
        .configure_mesh()
        .disable_x_mesh()
        .y_desc("Value")
        .x_desc("Quarter")
        .x_label_formatter(&|x| {
            if *x < data.len() {
                data[*x].0.to_string()
            } else {
                String::new()
            }
        })
        .label_style(("sans-serif", 15).into_font().color(&config.theme.foreground_color()))
        .draw()?;
    
    let color = config.theme.plot_colors()[0];
    
    chart.draw_series(
        data.iter().enumerate().map(|(idx, (_label, value))| {
            Rectangle::new([(idx, 0.0), (idx, *value)], color.filled())
        }),
    )?;
    
    root.present()?;
    Ok(())
}

/// Create a scatter plot
fn create_scatter_plot(path: &Path, config: &PlotTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
    root.fill(&config.theme.background_color())?;
    
    // Generate scatter data with some correlation
    let data: Vec<(f64, f64)> = (0..200)
        .map(|i| {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let x = i as f64 / 10.0;
            let y = x * 1.5 + rng.gen_range(-3.0..3.0);
            (x, y)
        })
        .collect();
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&config.name, ("sans-serif", 40).into_font().color(&config.theme.foreground_color()))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..20f64, 0f64..35f64)?;
    
    chart
        .configure_mesh()
        .x_desc("Variable X")
        .y_desc("Variable Y")
        .label_style(("sans-serif", 15).into_font().color(&config.theme.foreground_color()))
        .axis_style(&config.theme.foreground_color())
        .light_line_style(&config.theme.grid_color())
        .draw()?;
    
    let color = config.theme.plot_colors()[2];
    
    chart.draw_series(
        data.iter().map(|point| Circle::new(*point, 3, color.filled())),
    )?;
    
    root.present()?;
    Ok(())
}

/// Create a histogram
fn create_histogram(path: &Path, config: &PlotTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
    root.fill(&config.theme.background_color())?;
    
    let data = generate_histogram_data();
    
    // Create bins
    let num_bins = 30;
    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bin_width = (max_val - min_val) / num_bins as f64;
    
    let mut bins = vec![0; num_bins];
    for value in &data {
        let bin_idx = ((value - min_val) / bin_width).floor() as usize;
        if bin_idx < num_bins {
            bins[bin_idx] += 1;
        }
    }
    
    let max_count = *bins.iter().max().unwrap() as f64;
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&config.name, ("sans-serif", 40).into_font().color(&config.theme.foreground_color()))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_val..max_val, 0f64..max_count * 1.1)?;
    
    chart
        .configure_mesh()
        .x_desc("Value")
        .y_desc("Frequency")
        .label_style(("sans-serif", 15).into_font().color(&config.theme.foreground_color()))
        .axis_style(&config.theme.foreground_color())
        .light_line_style(&config.theme.grid_color())
        .draw()?;
    
    let color = config.theme.plot_colors()[1];
    
    chart.draw_series(
        bins.iter().enumerate().map(|(idx, &count)| {
            let x0 = min_val + idx as f64 * bin_width;
            let x1 = x0 + bin_width;
            Rectangle::new([(x0, 0.0), (x1, count as f64)], color.filled())
        }),
    )?;
    
    root.present()?;
    Ok(())
}

/// Create an area plot
fn create_area_plot(path: &Path, config: &PlotTestConfig, data: &[Vec<(f64, f64)>]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
    root.fill(&config.theme.background_color())?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&config.name, ("sans-serif", 40).into_font().color(&config.theme.foreground_color()))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..6f64, 0f64..35f64)?;
    
    chart
        .configure_mesh()
        .x_desc("Time")
        .y_desc("Value")
        .label_style(("sans-serif", 15).into_font().color(&config.theme.foreground_color()))
        .axis_style(&config.theme.foreground_color())
        .light_line_style(&config.theme.grid_color())
        .draw()?;
    
    let colors = config.theme.plot_colors();
    
    // Draw first series as area
    if let Some(series) = data.first() {
        let color = colors[0];
        chart.draw_series(
            AreaSeries::new(
                series.iter().cloned(),
                0.0,
                &color.mix(0.3),
            ).border_style(ShapeStyle::from(&color).stroke_width(2)),
        )?;
    }
    
    root.present()?;
    Ok(())
}

/// Export a plot in multiple formats
fn export_plot(base_path: &Path, config: &PlotTestConfig, data: &[Vec<(f64, f64)>]) -> Result<(), Box<dyn std::error::Error>> {
    // Create PNG
    let png_path = base_path.with_extension("png");
    match config.plot_type {
        PlotType::Line => create_line_plot(&png_path, config, data)?,
        PlotType::Bar => create_bar_plot(&png_path, config)?,
        PlotType::Scatter => create_scatter_plot(&png_path, config)?,
        PlotType::Histogram => create_histogram(&png_path, config)?,
        PlotType::Area => create_area_plot(&png_path, config, data)?,
    }
    
    // Create SVG (modify config for SVG backend)
    let svg_path = base_path.with_extension("svg");
    let svg_backend = SVGBackend::new(&svg_path, (config.width, config.height));
    let root = svg_backend.into_drawing_area();
    root.fill(&config.theme.background_color())?;
    
    // For SVG, we'll just create a simple version to show it works
    let mut chart = ChartBuilder::on(&root)
        .caption(&format!("{} (SVG)", config.name), ("sans-serif", 40).into_font().color(&config.theme.foreground_color()))
        .margin(20)
        .build_cartesian_2d(0f64..10f64, 0f64..10f64)?;
    
    chart.configure_mesh()
        .label_style(("sans-serif", 15).into_font().color(&config.theme.foreground_color()))
        .draw()?;
    
    root.present()?;
    
    println!("✅ Exported: {}", base_path.display());
    Ok(())
}

/// Main function to generate all test plots
pub fn generate_all_test_plots(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // Generate test data
    let multi_series = generate_multi_series();
    
    // Define all test configurations
    let plot_types = vec![
        PlotType::Line,
        PlotType::Bar,
        PlotType::Scatter,
        PlotType::Histogram,
        PlotType::Area,
    ];
    
    let themes = vec![
        ("light", Theme::Light),
        ("dark", Theme::Dark),
    ];
    
    let legend_options = vec![
        ("with_legend", true),
        ("no_legend", false),
    ];
    
    let sizes = vec![
        ("default", 800, 600),
        ("wide", 1200, 600),
        ("square", 800, 800),
    ];
    
    // Generate plots for all combinations
    for plot_type in &plot_types {
        for (theme_name, theme) in &themes {
            for (legend_name, with_legend) in &legend_options {
                for (size_name, width, height) in &sizes {
                    let config = PlotTestConfig {
                        name: format!("{:?} Plot", plot_type),
                        plot_type: *plot_type,
                        with_legend: *with_legend,
                        theme: *theme,
                        width: *width,
                        height: *height,
                    };
                    
                    let filename = format!(
                        "{:?}_{}_{}_{}", 
                        plot_type, 
                        theme_name, 
                        legend_name,
                        size_name
                    ).to_lowercase();
                    
                    let base_path = output_dir.join(filename);
                    export_plot(&base_path, &config, &multi_series)?;
                }
            }
        }
    }
    
    // Create an index HTML file to view all plots
    create_index_html(output_dir)?;
    
    println!("\n🎉 All plots generated successfully!");
    println!("📁 Output directory: {}", output_dir.display());
    println!("🌐 Open {}/index.html to view all plots", output_dir.display());
    
    Ok(())
}

/// Create an HTML index file to easily view all generated plots
fn create_index_html(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Pika Plot Export Test Results</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            margin: 20px;
            background: #f5f5f5;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
        }
        h1, h2, h3 { 
            color: #333; 
        }
        .plot-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        .plot-item {
            background: white;
            border: 1px solid #ddd;
            border-radius: 8px;
            padding: 15px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .plot-item img {
            width: 100%;
            height: auto;
            border: 1px solid #eee;
        }
        .plot-title {
            font-weight: bold;
            margin-bottom: 10px;
            color: #555;
        }
        .theme-section {
            margin: 40px 0;
        }
        .controls {
            margin: 20px 0;
            padding: 15px;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        button {
            padding: 8px 16px;
            margin: 5px;
            border: none;
            border-radius: 4px;
            background: #007bff;
            color: white;
            cursor: pointer;
        }
        button:hover {
            background: #0056b3;
        }
        .hidden {
            display: none;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 Pika Plot Export Test Results</h1>
        <p>Generated plot exports with various configurations for visual verification.</p>
        
        <div class="controls">
            <h3>Filter Controls:</h3>
            <button onclick="showAll()">Show All</button>
            <button onclick="filterByTheme('light')">Light Theme Only</button>
            <button onclick="filterByTheme('dark')">Dark Theme Only</button>
            <button onclick="filterByType('line')">Line Plots</button>
            <button onclick="filterByType('bar')">Bar Plots</button>
            <button onclick="filterByType('scatter')">Scatter Plots</button>
            <button onclick="filterByType('histogram')">Histograms</button>
            <button onclick="filterByType('area')">Area Plots</button>
        </div>
"#);
    
    // Group plots by type and theme
    let plot_types = vec!["line", "bar", "scatter", "histogram", "area"];
    let themes = vec!["light", "dark"];
    
    for plot_type in &plot_types {
        html.push_str(&format!("\n<div class='theme-section' data-type='{}'>\n", plot_type));
        html.push_str(&format!("<h2>{} Plots</h2>\n", plot_type.chars().next().unwrap().to_uppercase().to_string() + &plot_type[1..]));
        
        for theme in &themes {
            html.push_str(&format!("<h3>{} Theme</h3>\n", theme.chars().next().unwrap().to_uppercase().to_string() + &theme[1..]));
            html.push_str("<div class='plot-grid'>\n");
            
            // Add plots for this type and theme
            let patterns = vec![
                ("With Legend (Default Size)", format!("{}_{}_with_legend_default.png", plot_type, theme)),
                ("No Legend (Default Size)", format!("{}_{}_no_legend_default.png", plot_type, theme)),
                ("With Legend (Wide)", format!("{}_{}_with_legend_wide.png", plot_type, theme)),
                ("With Legend (Square)", format!("{}_{}_with_legend_square.png", plot_type, theme)),
            ];
            
            for (title, filename) in patterns {
                html.push_str(&format!(r#"
                <div class="plot-item" data-theme="{}" data-type="{}">
                    <div class="plot-title">{}</div>
                    <img src="{}" alt="{}" />
                </div>
"#, theme, plot_type, title, filename, title));
            }
            
            html.push_str("</div>\n");
        }
        html.push_str("</div>\n");
    }
    
    html.push_str(r#"
    </div>
    
    <script>
        function showAll() {
            document.querySelectorAll('.plot-item, .theme-section').forEach(el => {
                el.classList.remove('hidden');
            });
        }
        
        function filterByTheme(theme) {
            document.querySelectorAll('.plot-item').forEach(el => {
                if (el.dataset.theme === theme) {
                    el.classList.remove('hidden');
                } else {
                    el.classList.add('hidden');
                }
            });
            document.querySelectorAll('.theme-section').forEach(el => {
                el.classList.remove('hidden');
            });
        }
        
        function filterByType(type) {
            document.querySelectorAll('.theme-section').forEach(el => {
                if (el.dataset.type === type) {
                    el.classList.remove('hidden');
                } else {
                    el.classList.add('hidden');
                }
            });
            document.querySelectorAll('.plot-item').forEach(el => {
                el.classList.remove('hidden');
            });
        }
    </script>
</body>
</html>
"#);
    
    fs::write(output_dir.join("index.html"), html)?;
    Ok(())
} 