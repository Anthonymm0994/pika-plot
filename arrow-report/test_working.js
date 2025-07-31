const fs = require('fs');
const http = require('http');

console.log('🧪 Testing Working Arrow File...');

// Test 1: Check if the HTML file exists and has correct structure
function testHTMLFile() {
    console.log('1. Testing HTML file structure...');
    
    if (!fs.existsSync('working_arrow_test.html')) {
        console.log('❌ working_arrow_test.html not found');
        return false;
    }
    
    const content = fs.readFileSync('working_arrow_test.html', 'utf8');
    
    // Check for required libraries
    const hasArrow = content.includes('cdn.jsdelivr.net/npm/apache-arrow@21.0.0/Arrow.es2015.min.js');
    const hasPlot = content.includes('@observablehq/plot@0.6.17/dist/plot.umd.min.js');
    
    if (!hasArrow) {
        console.log('❌ Missing Apache Arrow library');
        return false;
    }
    
    if (!hasPlot) {
        console.log('❌ Missing Observable Plot library');
        return false;
    }
    
    // Check for parsing logic
    const hasParsingLogic = content.includes('Arrow.Table.from') || 
                           content.includes('Arrow.tableFrom') || 
                           content.includes('Arrow.read');
    
    if (!hasParsingLogic) {
        console.log('❌ Missing Arrow parsing logic');
        return false;
    }
    
    // Check for plotting logic
    const hasPlottingLogic = content.includes('Plot.plot');
    
    if (!hasPlottingLogic) {
        console.log('❌ Missing plotting logic');
        return false;
    }
    
    console.log('✅ HTML file structure is correct');
    return true;
}

// Test 2: Check if Arrow files exist
function testArrowFiles() {
    console.log('2. Testing Arrow files...');
    
    const files = ['test_data.arrow', 'time_series_data.arrow'];
    let allExist = true;
    
    files.forEach(file => {
        if (fs.existsSync(file)) {
            const stats = fs.statSync(file);
            console.log(`✅ ${file} exists (${stats.size} bytes)`);
        } else {
            console.log(`❌ ${file} not found`);
            allExist = false;
        }
    });
    
    return allExist;
}

// Test 3: Check if server is accessible
function testServer() {
    console.log('3. Testing server...');
    
    return new Promise((resolve) => {
        http.get('http://localhost:8001/working_arrow_test.html', (response) => {
            if (response.statusCode === 200) {
                console.log('✅ Server is running and file is accessible');
                resolve(true);
            } else {
                console.log(`❌ Server returned HTTP ${response.statusCode}`);
                resolve(false);
            }
        }).on('error', (err) => {
            console.log(`❌ Server not accessible: ${err.message}`);
            console.log('💡 Make sure to run: cd arrow-report && python -m http.server 8001');
            resolve(false);
        });
    });
}

// Test 4: Check CDN libraries
function testCDNLibraries() {
    console.log('4. Testing CDN libraries...');
    
    const libraries = [
        {
            name: 'Apache Arrow',
            url: 'https://cdn.jsdelivr.net/npm/apache-arrow@21.0.0/Arrow.es2015.min.js'
        },
        {
            name: 'Observable Plot',
            url: 'https://unpkg.com/@observablehq/plot@0.6.17/dist/plot.umd.min.js'
        }
    ];
    
    const promises = libraries.map(lib => {
        return new Promise((resolve) => {
            const https = require('https');
            https.get(lib.url, (response) => {
                if (response.statusCode === 200) {
                    let data = '';
                    response.on('data', (chunk) => data += chunk);
                    response.on('end', () => {
                        console.log(`✅ ${lib.name} accessible (${data.length} bytes)`);
                        resolve(true);
                    });
                } else {
                    console.log(`❌ ${lib.name} not accessible (HTTP ${response.statusCode})`);
                    resolve(false);
                }
            }).on('error', (err) => {
                console.log(`❌ ${lib.name} request failed: ${err.message}`);
                resolve(false);
            });
        });
    });
    
    return Promise.all(promises).then(results => {
        const allAccessible = results.every(result => result);
        return allAccessible;
    });
}

// Run all tests
async function runAllTests() {
    try {
        const htmlValid = testHTMLFile();
        const arrowFilesExist = testArrowFiles();
        const serverRunning = await testServer();
        const cdnLibrariesAccessible = await testCDNLibraries();
        
        console.log('\n🎯 Test Summary');
        console.log('===============');
        console.log(`HTML File: ${htmlValid ? '✅ Valid' : '❌ Invalid'}`);
        console.log(`Arrow Files: ${arrowFilesExist ? '✅ Exist' : '❌ Missing'}`);
        console.log(`Server: ${serverRunning ? '✅ Running' : '❌ Not running'}`);
        console.log(`CDN Libraries: ${cdnLibrariesAccessible ? '✅ Accessible' : '❌ Not accessible'}`);
        
        const allPassed = htmlValid && arrowFilesExist && serverRunning && cdnLibrariesAccessible;
        
        if (allPassed) {
            console.log('\n🚀 All tests passed! The file should work.');
            console.log('\n📋 To test:');
            console.log('1. Open http://localhost:8001/working_arrow_test.html in your browser');
            console.log('2. Select test_data.arrow or time_series_data.arrow');
            console.log('3. Click "Load & Parse Arrow File"');
            console.log('4. Click "Create Plot"');
        } else {
            console.log('\n❌ Some tests failed. Please fix the issues above.');
            process.exit(1);
        }
        
    } catch (error) {
        console.log('\n❌ Test suite failed:', error.message);
        process.exit(1);
    }
}

runAllTests(); 