babel="../../node_modules/.bin/babel"

find build -type f -delete
mkdir -p build
${babel} steward-node/responder.ts -o build/responder.js --presets=@babel/preset-typescript
cd transient-node
../../../node_modules/.bin/webpack --config ./webpack.config.js -p
