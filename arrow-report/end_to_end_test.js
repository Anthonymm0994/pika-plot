const fs = require('fs');
const https = require('https');

console.log('🧪 End-to-End Arrow File Test Suite');
console.log('=====================================\n');

// Test 1: Verify Arrow files exist and are valid
function testArrowFiles() {
    console.log('1. Testing Arrow Files...');
    
    const files = ['test_data.arrow', 'time_series_data.arrow'];
    let allValid = true;
    
    files.forEach(file => {
        if (fs.existsSync(file)) {
            const stats = fs.statSync(file);
            const buffer = fs.readFileSync(file);
            
            console.log(`   ✅ ${file} exists (${stats.size} bytes)`);
            
            // Check Arrow magic bytes (ARROW1)
            const magicBytes = buffer.slice(0, 6);
            const magicString = String.fromCharCode(...magicBytes);
            
            if (magicString === 'ARROW1') {
                console.log(`   ✅ Valid Arrow format (magic: ${magicString})`);
            } else {
                console.log(`   ❌ Invalid Arrow format (magic: ${magicString})`);
                allValid = false;
            }
            
            // Check file size is reasonable
            if (stats.size > 100 && stats.size < 1000000) {
                console.log(`   ✅ File size reasonable (${stats.size} bytes)`);
            } else {
                console.log(`   ⚠️  File size unusual (${stats.size} bytes)`);
            }
            
        } else {
            console.log(`   ❌ ${file} not found`);
            allValid = false;
        }
        console.log('');
    });
    
    return allValid;
}

// Test 2: Verify CDN libraries are accessible
function testCDNLibraries() {
    console.log('2. Testing CDN Libraries...');
    
    const libraries = [
        {
            name: 'Apache Arrow',
            url: 'https://cdn.jsdelivr.net/npm/apache-arrow@21.0.0/Arrow.es2015.min.js'
        },
        {
            name: 'Arquero',
            url: 'https://unpkg.com/arquero@5.3.0/dist/arquero.min.js'
        },
        {
            name: 'Observable Plot',
            url: 'https://unpkg.com/@observablehq/plot@0.6.17/dist/plot.umd.min.js'
        },
        {
            name: 'D3',
            url: 'https://unpkg.com/d3@7.8.5/dist/d3.min.js'
        }
    ];
    
    const promises = libraries.map(lib => {
        return new Promise((resolve) => {
            https.get(lib.url, (response) => {
                if (response.statusCode === 200) {
                    let data = '';
                    response.on('data', (chunk) => data += chunk);
                    response.on('end', () => {
                        console.log(`   ✅ ${lib.name} accessible (${data.length} bytes)`);
                        resolve(true);
                    });
                } else {
                    console.log(`   ❌ ${lib.name} not accessible (HTTP ${response.statusCode})`);
                    resolve(false);
                }
            }).on('error', (err) => {
                console.log(`   ❌ ${lib.name} request failed: ${err.message}`);
                resolve(false);
            });
        });
    });
    
    return Promise.all(promises).then(results => {
        const allAccessible = results.every(result => result);
        console.log('');
        return allAccessible;
    });
}

// Test 3: Verify HTML files have correct structure
function testHTMLFiles() {
    console.log('3. Testing HTML Files...');
    
    const htmlFiles = [
        { name: 'actual_test.html', required: true },
        { name: 'comprehensive_test.html', required: true },
        { name: 'index.html', required: false },
        { name: 'test.html', required: false }
    ];
    
    let allValid = true;
    
    htmlFiles.forEach(file => {
        if (fs.existsSync(file.name)) {
            const content = fs.readFileSync(file.name, 'utf8');
            
            console.log(`   ✅ ${file.name} exists`);
            
            // Check for required libraries
            const hasArrow = content.includes('apache-arrow@21.0.0/Arrow.es2015.min.js');
            const hasArquero = content.includes('arquero@5.3.0/dist/arquero.min.js');
            const hasPlot = content.includes('@observablehq/plot@0.6.17/dist/plot.umd.min.js');
            const hasD3 = content.includes('d3@7.8.5/dist/d3.min.js');
            
            if (hasArrow) console.log(`   ✅ Has Apache Arrow library`);
            else { console.log(`   ❌ Missing Apache Arrow library`); allValid = false; }
            
            if (hasArquero) console.log(`   ✅ Has Arquero library`);
            else { console.log(`   ❌ Missing Arquero library`); allValid = false; }
            
            if (hasPlot) console.log(`   ✅ Has Observable Plot library`);
            else { console.log(`   ❌ Missing Observable Plot library`); allValid = false; }
            
            if (hasD3) console.log(`   ✅ Has D3 library`);
            else { console.log(`   ❌ Missing D3 library`); allValid = false; }
            
            // Check for parsing logic (either inline or in script files)
            const hasParsingLogic = content.includes('Arrow.Table.from') || 
                                  content.includes('Arrow.tableFrom') || 
                                  content.includes('Arrow.read') ||
                                  content.includes('js/loader.js');
            
            if (hasParsingLogic) console.log(`   ✅ Has Arrow parsing logic`);
            else { console.log(`   ❌ Missing Arrow parsing logic`); allValid = false; }
            
            // Check for plotting logic (either inline or in script files)
            const hasPlottingLogic = content.includes('Plot.plot') ||
                                   content.includes('js/plot.js');
            
            if (hasPlottingLogic) console.log(`   ✅ Has plotting logic`);
            else { console.log(`   ❌ Missing plotting logic`); allValid = false; }
            
        } else if (file.required) {
            console.log(`   ❌ ${file.name} not found (required)`);
            allValid = false;
        } else {
            console.log(`   ⚠️  ${file.name} not found (optional)`);
        }
        console.log('');
    });
    
    return allValid;
}

// Test 4: Verify test scripts
function testScripts() {
    console.log('4. Testing Scripts...');
    
    const scripts = [
        { name: 'test_real_arrow.js', required: true },
        { name: 'create_test_arrow.py', required: false }
    ];
    
    let allValid = true;
    
    scripts.forEach(script => {
        if (fs.existsSync(script.name)) {
            const content = fs.readFileSync(script.name, 'utf8');
            console.log(`   ✅ ${script.name} exists`);
            
            if (script.name.includes('test_real_arrow.js')) {
                const hasArrowFileCheck = content.includes('test_data.arrow') && content.includes('time_series_data.arrow');
                const hasHTMLCheck = content.includes('actual_test.html');
                
                if (hasArrowFileCheck) console.log(`   ✅ Has Arrow file checks`);
                else { console.log(`   ❌ Missing Arrow file checks`); allValid = false; }
                
                if (hasHTMLCheck) console.log(`   ✅ Has HTML file checks`);
                else { console.log(`   ❌ Missing HTML file checks`); allValid = false; }
            }
            
        } else if (script.required) {
            console.log(`   ❌ ${script.name} not found (required)`);
            allValid = false;
        } else {
            console.log(`   ⚠️  ${script.name} not found (optional)`);
        }
        console.log('');
    });
    
    return allValid;
}

// Test 5: Verify server is running
function testServer() {
    console.log('5. Testing Server...');
    
    return new Promise((resolve) => {
        const http = require('http');
        http.get('http://localhost:8001/actual_test.html', (response) => {
            if (response.statusCode === 200) {
                console.log('   ✅ Server is running on port 8001');
                console.log('   ✅ actual_test.html is accessible');
                resolve(true);
            } else {
                console.log(`   ❌ Server returned HTTP ${response.statusCode}`);
                resolve(false);
            }
        }).on('error', (err) => {
            console.log(`   ❌ Server not accessible: ${err.message}`);
            console.log('   💡 Make sure to run: cd arrow-report && python -m http.server 8001');
            resolve(false);
        });
    });
}

// Run all tests
async function runAllTests() {
    try {
        const arrowFilesValid = testArrowFiles();
        const cdnLibrariesValid = await testCDNLibraries();
        const htmlFilesValid = testHTMLFiles();
        const scriptsValid = testScripts();
        const serverRunning = await testServer();
        
        console.log('🎯 Test Summary');
        console.log('===============');
        console.log(`Arrow Files: ${arrowFilesValid ? '✅ Valid' : '❌ Invalid'}`);
        console.log(`CDN Libraries: ${cdnLibrariesValid ? '✅ Accessible' : '❌ Not accessible'}`);
        console.log(`HTML Files: ${htmlFilesValid ? '✅ Valid' : '❌ Invalid'}`);
        console.log(`Scripts: ${scriptsValid ? '✅ Valid' : '❌ Invalid'}`);
        console.log(`Server: ${serverRunning ? '✅ Running' : '❌ Not running'}`);
        
        const allPassed = arrowFilesValid && cdnLibrariesValid && htmlFilesValid && scriptsValid && serverRunning;
        
        if (allPassed) {
            console.log('\n🚀 All tests passed! The project is ready for use.');
            console.log('\n📋 Next Steps:');
            console.log('1. Open http://localhost:8001/actual_test.html in your browser');
            console.log('2. Select an Arrow file (test_data.arrow or time_series_data.arrow)');
            console.log('3. Click "Load & Parse Arrow File"');
            console.log('4. Click "Create Plot" to see the visualization');
            console.log('5. For comprehensive testing, use http://localhost:8001/comprehensive_test.html');
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