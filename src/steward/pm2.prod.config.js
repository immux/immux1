// For pm2

module.exports = {
    apps: [
        {
            name: 'immuxdb',
            script: 'build/immuxdb',
        },
        {
            name: 'steward',
            script: 'build/steward/steward.js',
        },
    ]
}
