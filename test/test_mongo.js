const path = require('path')
const exec = require('child_process').exec
const glob = require('glob')

const MONGO_SOURCE_ROOT = path.join(__dirname, `mongo`)
const TEST_ROOT = path.join(MONGO_SOURCE_ROOT, 'jstests')
const CONCURRENT_TESTS = 16

function execAsync(command) {
    return new Promise((resolve, reject) => {
        exec(command, (error, stdout, stderr) => {
            if (stderr) {
                reject(stderr)
            }
            else if (error) {
                reject(error)
            }
            else {
                resolve(stdout)
            }
        })
    })
}

async function testMongo() {
    process.chdir(MONGO_SOURCE_ROOT);
    let testBase = TEST_ROOT;
    if (process.argv[2]) {
        testBase = path.join(testBase, process.argv[2])
    }
    const testFiles = glob.sync(`${testBase}/**/*.js`)
    let executedCount = 0;
    let okCount = 0;
    let totalTestCount = testFiles.length;
    console.info(`Running ${totalTestCount} tests from ${testBase}`)

    let failedTests = []

    let finalReported = false

    async function executeTest() {
        while (testFiles.length) {
            let testFile = testFiles[0]
            testFiles.splice(0, 1)
            try {
                await execAsync(`mongo ${testFile}`)
                executedCount += 1;
                okCount += 1
                console.info(`OK [${okCount}|${executedCount}|${totalTestCount}] - ${testFile}`)
            }
            catch (error) {
                executedCount += 1;
                console.error(`Fail [${okCount}|${executedCount}|${totalTestCount}] - ${testFile} - error ${error}`)
                failedTests.push(testFile)
            }
        }
        if (!finalReported) {
            finalReported = true
            if (failedTests.length) {
                console.info('Failed tests:', failedTests)
            }
            console.info(`Total ${totalTestCount} tests, ${okCount} passed, `
                         + `passing rate ${Math.round((okCount / totalTestCount) * 1000) / 10}%`)
        }
    }

    for (let i = 0; i < CONCURRENT_TESTS; i += 1) {
        executeTest()
    }
}

!testMongo()
