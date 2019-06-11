const path = require('path')
const HtmlWebpackPlugin = require('html-webpack-plugin')
const HtmlWebpackInlineSourcePlugin = require('html-webpack-inline-source-plugin')

module.exports = {
    entry: path.join(__dirname, 'web.tsx'),
    mode: 'development',

    output: {
        path: path.resolve(__dirname, '../build/'),
        publicPath: '/',
        filename: '[name].[chunkhash].js'
    },

    resolve: {
        extensions: ['.ts', '.tsx', '.js', '.css']
    },

    module: {
        rules: [
            {
                test: /\.tsx?$/,
                loader: 'ts-loader',
                options: {
                    configFile: './tsconfig.json',
                }
            },
        ]
    },

    devServer: {
        contentBase: '/',
        historyApiFallback: true,
        port: 9000,
        allowedHosts: [
            ".foldr.test",
        ]
    },

    plugins: [
        new HtmlWebpackPlugin({
            template: path.join(__dirname, 'index.html'),
            filename: 'index.html',
            inject: 'body',
            inlineSource: '.(js|css)$',
        }),
        new HtmlWebpackInlineSourcePlugin()
    ]
}
