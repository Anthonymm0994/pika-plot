const fs = require('fs');

// Test if we can read the Arrow files
function testArrowFiles() {
    console.log('Testing Arrow files...\n');
    
    const files = ['test_data.arrow', 'time_series_data.arrow'];
    
    files.forEach(file => {
        if (fs.existsSync(file)) {
            const stats = fs.statSync(file);
            const buffer = fs.readFileSync(file);
            
            console.log(`✅ ${file} exists (${stats.size} bytes)`);
            console.log(`   First 32 bytes: [${Array.from(buffer.slice(0, 32)).join(', ')}]`);
            
            // Check if it looks like an Arrow file (should start with specific bytes)
            const firstBytes = buffer.slice(0, 8);
            console.log(`   Magic bytes: [${Array.from(firstBytes).join(', ')}]`);
            
            if (firstBytes[0] === 0xFF && firstBytes[1] === 0x52 && firstBytes[2] === 0x52 && firstBytes[3] === 0x00) {
                console.log(`   ✅ Looks like a valid Arrow file`);
            } else {
                console.log(`   ⚠️  May not be a valid Arrow file format`);
            }
            
        } else {
            console.log(`❌ ${file} not found`);
        }
        console.log('');
    });
}

// Test the HTML page
function testHTMLPage() {
    console.log('Testing HTML page...\n');
    
    const htmlFile = 'actual_test.html';
    if (fs.existsSync(htmlFile)) {
        const content = fs.readFileSync(htmlFile, 'utf8');
        
        // Check if it has the correct Arrow library URL
        if (content.includes('cdn.jsdelivr.net/npm/apache-arrow@21.0.0/Arrow.es2015.min.js')) {
            console.log('✅ HTML page has correct Arrow library URL');
        } else {
            console.log('❌ HTML page has incorrect Arrow library URL');
        }
        
        // Check if it has Plot library
        if (content.includes('@observablehq/plot@0.6.17/dist/plot.umd.min.js')) {
            console.log('✅ HTML page has Plot library');
        } else {
            console.log('❌ HTML page missing Plot library');
        }
        
        // Check if it has the parsing logic
        if (content.includes('Arrow.Table.from')) {
            console.log('✅ HTML page has Arrow parsing logic');
        } else {
            console.log('❌ HTML page missing Arrow parsing logic');
        }
        
        // Check if it has plotting logic
        if (content.includes('Plot.plot')) {
            console.log('✅ HTML page has plotting logic');
        } else {
            console.log('❌ HTML page missing plotting logic');
        }
        
    } else {
        console.log(`❌ ${htmlFile} not found`);
    }
}

// Run tests
console.log('🧪 Real Arrow File Test');
console.log('========================\n');

testArrowFiles();
testHTMLPage();

console.log('🎯 Instructions:');
console.log('1. Open http://localhost:8001/actual_test.html in your browser');
console.log('2. Select one of the Arrow files (test_data.arrow or time_series_data.arrow)');
console.log('3. Click "Load & Parse Arrow File"');
console.log('4. If successful, click "Create Plot"');
console.log('5. Check browser console for detailed logs'); 