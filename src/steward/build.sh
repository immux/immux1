root=$(pwd)
TSC="./node_modules/.bin/tsc"
CARGO="/home/deploy/.cargo/bin/cargo"

## Clean up
find build -type f -delete

## Build foldr
cd projects/foldr
./build.sh
cp -r build ${root}/build/foldr
cd ${root}

## Build steward
${TSC} -p tsconfig.json

## Build ImmuxDB
${CARGO} build --release -j 1
cp ../../target/release/immuxdb build/
