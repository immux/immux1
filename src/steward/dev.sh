PM2='./node_modules/.bin/pm2'
TSC='./node_modules/.bin/tsc'

root=$(pwd)

# Build database
cargo build

# Build steward
${TSC} -p tsconfig.json

# nginx
cd projects/foldr
killall nginx
nginx -p $(pwd) -c ./nginx.dev.conf
cd ${root}

${PM2} start pm2.dev.config.js

echo "To exit, run\n${PM2} kill"
