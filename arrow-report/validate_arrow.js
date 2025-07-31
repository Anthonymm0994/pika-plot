const fs = require('fs');
const { tableFromIPC } = require('apache-arrow');

console.log('🧪 Validating Arrow File Parsing...');

function testArrowFile(filename) {
    console.log(`\n📁 Testing ${filename}...`);
    
    try {
        if (!fs.existsSync(filename)) {
            console.log(`❌ File ${filename} not found`);
            return false;
        }
        
        const arrowBuffer = fs.readFileSync(filename);
        console.log(`✅ File loaded: ${arrowBuffer.length} bytes`);
        
        // Check for Arrow magic bytes
        const magicBytes = arrowBuffer.slice(0, 6).toString();
        if (magicBytes === 'ARROW1') {
            console.log('✅ Valid Arrow format (magic bytes: ARROW1)');
        } else {
            console.log(`⚠️  Magic bytes: ${magicBytes} (expected ARROW1)`);
        }
        
        // Parse using tableFromIPC
        const table = tableFromIPC(arrowBuffer);
        console.log(`✅ Parsed successfully with tableFromIPC`);
        console.log(`   Rows: ${table.numRows}`);
        console.log(`   Columns: ${table.numCols}`);
        console.log(`   Schema: ${table.schema.fields.map(f => `${f.name}(${f.type})`).join(', ')}`);
        
        // Convert to array
        const data = table.toArray();
        console.log(`✅ Converted to array: ${data.length} rows`);
        console.log(`   Sample data: ${JSON.stringify(data.slice(0, 2))}`);
        
        return true;
        
    } catch (error) {
        console.log(`❌ Error parsing ${filename}: ${error.message}`);
        return false;
    }
}

// Test both Arrow files
const files = ['simple_test.arrow', 'simple_timeseries.arrow'];
let allPassed = true;

files.forEach(file => {
    const success = testArrowFile(file);
    if (!success) {
        allPassed = false;
    }
});

if (allPassed) {
    console.log('\n🎉 All Arrow files parsed successfully!');
    console.log('The HTML file should work correctly with these files.');
} else {
    console.log('\n❌ Some files failed to parse.');
    process.exit(1);
} 