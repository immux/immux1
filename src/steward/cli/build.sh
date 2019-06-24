find build -type f -delete

TSC="./node_modules/.bin/tsc"
${TSC}
cp package.json build/
cp -r skeleton build/cli/
