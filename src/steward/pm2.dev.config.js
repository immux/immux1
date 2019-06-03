module.exports = {
    apps: [
        {
            name: 'immuxdb',
            script: '../../target/debug/immuxdb',
        },
        {
            name: 'steward',
            script: 'build/steward/steward.js',
            watch: true,
        },
        {
            name: 'foldr-fe-dev-server',
            script: './node_modules/.bin/webpack-dev-server',
            cwd: './projects/foldr/transient-node/'
        },
        {
            name: 'tsc',
            script: 'tsc',
            args: '-p tsconfig.json --watch',
        },
    ]
}
