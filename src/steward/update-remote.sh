cd /var/www/immux
#git checkout master
git pull
cd src/steward
npm install
./build.sh
pm2 restart all

# foldr
cd projects/foldr
./build.sh
../../node_modules/.bin/ts-node updateFoldr.ts
